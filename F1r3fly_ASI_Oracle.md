
# Oracle Brain Dump for ASI Project

## 1. Environments

Currently, we have three environments (clusters) for the ASI project:

-   dev
-   stg
-   prod

Each cluster is configured with 2 OCPUs and 6 GB of memory (these limits are chosen to optimize resource usage, but they are sufficient as a first step). Under each cluster, we have one node, and each node runs a single pod (the f1r3fly application). We perform deployments using Helm files - [f1r3fly helm](https://github.com/F1R3FLY-io/f1r3fly/tree/main/docker/helm).

----------

## 2. Configuration of the Local Environment for Remote Communication with Oracle Cloud (Logs, Cluster Status, Deployments, etc)

To successfully communicate with Oracle Cloud, you will need the proper credentials.

### How to Send Requests to Oracle?

To communicate with the clusters, you need:

-   A client (e.g., F1r3flyFS)
-   The Public IP address of each node (which you can obtain from the cluster information)
-   Port 30002 (this port is open, and future requests will be sent to it)

Public IPs of the clusters:

-   dev public IP: `"146.235.215.215"`
-   stg public IP: `"159.54.178.87"`
-   prod public IP: `"167.234.221.56"`

As an example, you can use `telnet` from the terminal to check if you can send a message to the backend deployed on Oracle. Since the clusters are public and have an open port (30002), you can try telnet to the dev environment:

`telnet 146.235.215.215 30002`

However, for proper communication through which you can deploy Rholang code, you will need your own client. For example, you can use the **F1r3flyFS** client - [git](https://github.com/F1R3FLY-io/F1R3FLYFS).

----------

### Oracle Cloud Sign-In

1.  Go to [Oracle Sign-In](https://www.oracle.com/cloud/sign-in.html).
2.  Provide the Cloud Account Name - `f1r3fly`.
3.  Provide your credentials (email, password).

----------

### Kubernetes Deployment Communication

We use Kubernetes for deployments, so for communication with the remote server, you will need **kubectl**. Refer to the official [Kubernetes documentation](https://kubernetes.io/docs/tasks/tools/) for installation and setup. You should see something like this after setup:
```
@kubectl version:

Client Version: v1.30.1 // client version
Server Version: v1.31.1 // server version
```
----------

### Local Configuration

Communication with the remote Kubernetes server is through **oci-cli**. If you want to perform deployments or check cluster information, you need to set up communication with Oracle using the following steps.

**1. Install or Upgrade oci-cli:**

`brew update && brew install oci-cli`

or if it's already installed:

`brew update && brew upgrade oci-cli`

**2. Set Up Local Configuration:**

Run the following command to configure your local environment:

`oci setup config`

This will generate the public and private keys and configure the `config` file. If you've already run this command before and have multiple profiles, you can run it again without overwriting your files. It will simply update the `config` with a new profile and create additional keys.

To configure the `config` file, you will need to log in to Oracle and retrieve these parameters:

-   `user OCID`
-   `tenancy OCID`
-   `Your region`

After successful configuration and key generation, your `config` file will look like this:

`[DEFAULT]
user=ocid1.user.oc1..aaaaaaaaljasfjgiuonldsqhmm3fstuhq6zyznm777tsvy5npfrdnv3ejqwe
fingerprint=04:40:f1:61:b3:77:e6:11:fb:11:11:d5:4c:fe:11:6a
key_file=/Users/nazaryanovets/.oci/oci_api_key.pem
tenancy=ocid1.tenancy.oc1..aaaaaaaaii0udpmtqhokgof6mqewbfipvbgu7ux4vvpm7gaghfnnuvgjqwe
region=us-sanjose-1
pass_phrase=example`

Where `key_file` is the path to your private key.

By default, you will find all necessary files in the `$HOME` directory under the `.oci` folder.

**3. Add the Public Key to Oracle Cloud:**

Copy your public key:

`cat $HOME/.oci/oci_api_key_public.pem | pbcopy`

Then, follow these steps:

1.  Log into Oracle Cloud -> Profile -> My Profile -> Resources -> API Keys
2.  Add API Keys -> Paste the public key -> Save

If all the previous steps are done correctly, run the following command from the terminal:

`oci os ns get`

You should see something like:

`{
"data": "axb4qezqa1q2"
}`

This confirms that you can now communicate with Oracle via `oci-cli`.

For more commands and capabilities of `oci-cli`, refer to the [Oracle documentation](https://docs.oracle.com/en-us/iaas/tools/oci-cli/3.39.1/oci_cli_docs/).

----------

For more information on the deployment process, check out the [Cloud Brain Dump](https://github.com/F1R3FLY-io/f1r3fly/blob/main/Cloud_brain_dump.md).