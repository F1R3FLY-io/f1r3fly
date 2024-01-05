#!/usr/bin/env bash
set -e

if [[ "$1" = "-h" || "$1" == "--help" ]]; then
  # echo "usage: $0 <docker> directory"
  echo "set DOCKER_ROOT env variable to directory location for override of f1refly/docker directory"
  exit 0
#elif [[ -z $1 ]]; then
#  : ${DOCKER_ROOT:="$1"}
#  echo DOCKER_ROOT: ${DOCKER_ROOT} -- variable set!
else
  : ${DOCKER_ROOT:="./docker"}
  echo DOCKER_ROOT: ${DOCKER_ROOT} -- variable set!
fi

set -m # Enable job control

# Trap SIGINT and SIGTERM to kill all child processes
trap 'for job in $(jobs -p); do kill $job; done; sleep 2; for job in $(jobs -p); do kill -9 $job; done' SIGINT SIGTERM
trap 'sleep 3; pkill java' SIGINT

# shellcheck disable=SC2012
rnode=$(ls -t ./node/target/scala-2.12/rnode-assembly-*.jar | head -1) 2>&1 # grab most recent jar file

runRnode="java --add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED -jar ${rnode}"
rNodeParams="--no-upnp --host localhost"

# jar file execution configuration:
Bootstrap="${runRnode} run ${rNodeParams} -c ${DOCKER_ROOT}/conf/bootstrap.conf --data-dir ${DOCKER_ROOT}/data/rnode.bootstrap"
printf "Starting: %s" "${Bootstrap}"

rnodePattern="rnode:"

# Run the command and pipe its output into a while loop
(${Bootstrap} &) | while read -r Line
do
    echo "$Line" # Print output to console
    if [[ "$Line" == *"$rnodePattern"* ]]; then
        bootstrapIdentity=$(echo "$Line" | awk '{ gsub(/\.$/, "", $13); print $13 }')                                                                                        
        Validator1="${runRnode} run ${rNodeParams} --bootstrap ${bootstrapIdentity} -c ${DOCKER_ROOT}/conf/validator1.conf --data-dir ${DOCKER_ROOT}/data/rnode.validator1 \
          -u -h 8081 -a 8082 -e 40001 -i 40002"
        IFS=' ' read -ra Val1Params <<< "${Validator1}"                                                                                                      
        printf "> Executing Validator 1 with:\n>> java %s" "${Val1Params[*]}\n" 
        # shellcheck disable=2048,2086
        (${Val1Params[*]} &)

        Validator2="${runRnode} run ${rNodeParams} --bootstrap ${bootstrapIdentity} -c ${DOCKER_ROOT}/conf/validator2.conf --data-dir ${DOCKER_ROOT}/data/rnode.validator2 \
          -u -h 8085 -a 8086 -e 40005 -i 40006"
        IFS=' ' read -ra Validator2Params <<< "${Validator2}"     
        # shellcheck disable=2048,2086
        ${Validator2Params[*]}
    fi
done

# Wait for all background jobs to finish
wait
