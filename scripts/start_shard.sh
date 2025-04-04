#!/usr/bin/env bash
set -e

if [[ "$1" = "-h" || "$1" == "--help" ]]; then
	# echo "usage: $0 <rchain.xmpl> directory"
	echo "set RCHAINXMPL env variable to directory location for override of ~/Downloads/rchain.xmpl/ directory"
	exit 0
#elif [[ -z $1 ]]; then
#  : ${RCHAINXMPL:="$1"}
#  echo RCHAINXMP: ${RCHAINXMPL} -- variable set!
else
	: ${RCHAINXMPL:="${HOME}/Downloads/rchain.xmpl"}
	echo RCHAINXMP: ${RCHAINXMPL} -- variable set!
fi

set -m # Enable job control

# Trap SIGINT and SIGTERM to kill all child processes
trap 'for job in $(jobs -p); do kill $job; done; sleep 2; for job in $(jobs -p); do kill -9 $job; done' SIGINT SIGTERM
trap 'sleep 3; pkill java' SIGINT

# shellcheck disable=SC2012
rnode=$(ls -t ./node/target/scala-2.12/rnode-assembly-*.jar | head -1) 2>&1 # grab most recent jar file

rnodePattern="rnode:"

baseParams="--add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED"
jarParam="-jar ${rnode}"
jnaParam="-Djna.library.path=./rspace++/target/release/"

syncrhonyParam="--synchrony-constraint-threshold=0.0"
# jar file execution configuration:
Bootstrap="java ${jnaParam} ${baseParams} ${jarParam} run -s --data-dir ${RCHAINXMPL}/node0/rnode"
printf "Starting: %s\n" "${Bootstrap}"

# Run the command and pipe its output into a while loop
(${Bootstrap} &) | while read -r Line; do
	echo "$Line" # Print output to console
	if [[ "$Line" == *"$rnodePattern"* ]]; then
		bootstrapIdentity=$(echo "$Line" | awk '{ gsub(/\.$/, "", $13); print $13 }')
		Validator1="${jnaParam} ${baseParams} ${jarParam} run ${syncrhonyParam} --data-dir ${RCHAINXMPL}/node1/rnode\
          -u -h 8081 -a 8082 -e 40001 -i 40002 --genesis-validator --bootstrap  ${bootstrapIdentity}"
		IFS=' ' read -ra Val1Params <<<"${Validator1}"
		printf "> Executing Validator 1 with:\n>> java %s" "${Val1Params[*]}"
		# shellcheck disable=2048,2086
		(java ${Val1Params[*]} &)

		Validator2="${jnaParam} ${baseParams} ${jarParam} run ${syncrhonyParam} --data-dir ${RCHAINXMPL}/node2/rnode\
          -u -h 8085 -a 8086 -e 40005 -i 40006 --genesis-validator --bootstrap  ${bootstrapIdentity}"
		IFS=' ' read -ra Validator2Params <<<"${Validator2}"
		printf "\n\n> Executing Validator 2 with:\n>> java %s\n" "${Validator2Params[*]}"
		# shellcheck disable=2048,2086
		java ${Validator2Params[*]}
	fi
done

# Wait for all background jobs to finish
wait
