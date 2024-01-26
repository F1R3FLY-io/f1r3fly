# See if there's a Docker process named "rancher-2.8.1," running or not
RANCHER_CID=`docker ps -a -q --no-trunc -f name=^/rancher\-2\.8\.1$`

# If there is a non-empty RANCHER_CID, stop the process. Used by the EXIT trap
function finish {
  if [ -n ${RANCHER_CID} ];
    then docker stop $RANCHER_CID
  fi
}

if [ -z ${RANCHER_CID} ];
  # If there's no rancher-2.8.1 process, running or not, run it
  then docker run \
    -e CATTLE_BOOTSTRAP_PASSWORD=snarf \
    --name rancher-2.8.1 \
    --privileged \
    -d \
    --restart=unless-stopped \
    -p 8080:80 \
    -p 8443:443 \
    rancher/rancher:v2.8.1
  # If there is such a process, running or not, start it. start is idempotent
  else docker start $RANCHER_CID
fi

# When the shell exits (hint: bring this script into an outer shell with .), stop Rancher
trap finish EXIT
