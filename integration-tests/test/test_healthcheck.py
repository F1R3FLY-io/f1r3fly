import pytest
import docker
import time
import json
import re
import os
import socket
import stat
from pathlib import Path

@pytest.fixture(scope="module")
def docker_client():
    return docker.from_env()

def is_port_free(port):
    """Check if a port is free on the host."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(1)
        return s.connect_ex(('localhost', port)) != 0

@pytest.fixture(scope="module")
def rnode_container(docker_client, tmpdir_factory):
    # Ensure required ports are free
    required_ports = [40401, 40403]
    for port in required_ports:
        if not is_port_free(port):
            raise RuntimeError(f"Port {port} is already in use. Please free it before running the test.")

    # Remove existing container if it exists
    container_name = "rnode_test_healthcheck"
    try:
        existing_container = docker_client.containers.get(container_name)
        existing_container.stop()
        existing_container.remove()
        print(f"Removed existing container: {container_name}")
    except docker.errors.NotFound:
        pass

    # Create unique temporary directories for logs and data
    rnode_logs_dir = tmpdir_factory.mktemp("rnode-logs")
    rnode_data_dir = tmpdir_factory.mktemp("rnode-data")

    # Set permissions to 777 for mounted directories
    os.chmod(str(rnode_logs_dir), stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)
    os.chmod(str(rnode_data_dir), stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)
    print(f"Set permissions for {rnode_logs_dir} to 777: {oct(os.stat(str(rnode_logs_dir)).st_mode)[-3:]}")
    print(f"Set permissions for {rnode_data_dir} to 777: {oct(os.stat(str(rnode_data_dir)).st_mode)[-3:]}")

    # Start the RNode container with volumes for logs and data
    try:
        container = docker_client.containers.run(
            "f1r3flyindustries/f1r3fly-scala-node:latest",
            detach=True,
            ports={"40401/tcp": 40401, "40403/tcp": 40403},
            name=container_name,
            volumes={
                str(rnode_logs_dir): {"bind": "/tmp", "mode": "rw"},
                str(rnode_data_dir): {"bind": "/var/lib/rnode", "mode": "rw"},
            },
            environment={"RNODE_PROFILE": "docker"}
        )
    except docker.errors.APIError as e:
        raise RuntimeError(f"Failed to start container: {str(e)}")

    # Wait for the container to become ready (up to 180 seconds)
    start_time = time.time()
    log_pattern = re.compile(r"Listening for traffic on|Server started|NodeEnvironment.*created|Ready", re.MULTILINE)
    try:
        while time.time() - start_time < 180:
            container.reload()
            logs = container.logs().decode("utf-8")
            if log_pattern.search(logs):
                # Wait for initial health check after service is ready
                health_start = time.time()
                while time.time() - health_start < 180:
                    # Use docker inspect to get health status
                    inspect_data = docker_client.api.inspect_container(container_name)
                    health_status = inspect_data.get("State", {}).get("Health", {}).get("Status", "")
                    health_logs = inspect_data.get("State", {}).get("Health", {}).get("Log", [])
                    print(f"Health status check at {time.time() - health_start:.1f}s: {health_status}")
                    if health_status == "healthy":
                        break
                    elif health_status == "unhealthy":
                        container_logs = container.logs().decode("utf-8")
                        # Check healthcheck log files
                        try:
                            grpcurl_log = container.exec_run("cat /tmp/grpcurl.log").output.decode("utf-8")
                            curl_log = container.exec_run("cat /tmp/curl.log").output.decode("utf-8")
                        except docker.errors.APIError:
                            grpcurl_log = curl_log = "Unable to read log files"
                        raise AssertionError(
                            f"Container reported unhealthy.\n"
                            f"Health logs: {json.dumps(health_logs, indent=2)}\n"
                            f"grpcurl.log: {grpcurl_log}\n"
                            f"curl.log: {curl_log}\n"
                            f"Container logs: {container_logs}"
                        )
                    time.sleep(5)  # Slower polling to avoid API issues
                else:
                    container_logs = container.logs().decode("utf-8")
                    # Check healthcheck log files
                    try:
                        grpcurl_log = container.exec_run("cat /tmp/grpcurl.log").output.decode("utf-8")
                        curl_log = container.exec_run("cat /tmp/curl.log").output.decode("utf-8")
                    except docker.errors.APIError:
                        grpcurl_log = curl_log = "Unable to read log files"
                    raise AssertionError(
                        f"Container did not become healthy within 180 seconds after service start.\n"
                        f"Health logs: {json.dumps(health_logs, indent=2)}\n"
                        f"grpcurl.log: {grpcurl_log}\n"
                        f"curl.log: {curl_log}\n"
                        f"Container logs: {container_logs}"
                    )
                break
            elif container.attrs.get("State", {}).get("Status") != "running":
                container_logs = container.logs().decode("utf-8")
                raise AssertionError(f"Container stopped unexpectedly.\nContainer logs: {container_logs}")
            time.sleep(5)  # Slower polling for readiness
        else:
            container_logs = container.logs().decode("utf-8")
            raise AssertionError(
                f"Container did not become ready within 180 seconds.\n"
                f"Container logs: {container_logs}"
            )
    except Exception as e:
        container.stop()
        container.remove()
        raise

    yield container

    # Cleanup
    try:
        container.stop()
        container.remove()
    except docker.errors.APIError:
        pass

def test_healthcheck(rnode_container):
    # Verify the container is healthy using docker inspect
    inspect_data = rnode_container.client.api.inspect_container(rnode_container.name)
    health_status = inspect_data.get("State", {}).get("Health", {}).get("Status", "")
    health_logs = inspect_data.get("State", {}).get("Health", {}).get("Log", [])
    container_logs = rnode_container.logs().decode("utf-8")
    assert health_status == "healthy", (
        f"Container healthcheck failed.\n"
        f"Health logs: {json.dumps(health_logs, indent=2)}\n"
        f"Container logs: {container_logs}"
    )
