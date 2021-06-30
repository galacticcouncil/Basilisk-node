#!/bin/bash

SCRIPT_NAME="fork-testing.sh"

function usage()
{
  echo ""
  echo "Start relaychain, parachain or both and genere genesis state and wasm"
    echo ""
    echo "Usage:"
    echo -e "  ./$SCRIPT_NAME --help"
    echo -e "  ./$SCRIPT_NAME --start-para"
    echo -e "  ./$SCRIPT_NAME --start-relay"
    echo -e "  ./$SCRIPT_NAME --start \n"
    echo -e "  ./$SCRIPT_NAME --stop-para"
    echo -e "  ./$SCRIPT_NAME --stop-relay"
    echo -e "  ./$SCRIPT_NAME --stop \n"
    echo -e "  ./$SCRIPT_NAME --set-collator-keys \n"
    echo -e "  ./$SCRIPT_NAME --get-para-state-wasm \n"

    echo -e "  Commands:"
    echo -e "    --start-para\t\t start parachain chain(cleared)"
    echo -e "    --start-relay\t\t start relaychain(cleared)"
    echo -e "    --start\t\t start parachain and relaychain(cleared)"
    echo -e "    --stop-para\t\t stop parachain chain"
    echo -e "    --stop-relay\t\t stop relaychain"
    echo -e "    --stop\t\t stop parachain and relaychain"
    echo -e "    --set-collator-keys\t\tkeys have to be in ./parachain/keys in json files"
    echo -e "    --get-para-state-wasm\t\tgenerate parachain state and wasm files"
    echo ""
}

PARA_ID=$(grep PARA_ID env | xargs)
PARA_ID=${PARA_ID#*=}
PARA_BINARY=$(grep PARA_BINARY env | xargs)
PARA_BINARY=${PARA_BINARY#*=}
PARACHAIN_SPEC=$(grep PARACHAIN_SPEC env | xargs)
PARACHAIN_SPEC=${PARACHAIN_SPEC#*=}

purge_relaychain_data () {
  echo "Purging RELAYCHAIN data..."

  rm -rf ./relaychain/alice/chains
  rm -rf ./relaychain/bob/chains
  rm -rf ./relaychain/charlie/chains
  rm -rf ./relaychain/dave/chains
}

purge_parachain_data () {
  echo "Purging PARACHAIN data..."

  rm -rf ./parachain/coll-01/chains
  rm -rf ./parachain/coll-01/polkadot
  rm -rf ./parachain/coll-02/chains
  rm -rf ./parachain/coll-02/polkadot
  rm -rf ./parachain/coll-03/chains
  rm -rf ./parachain/coll-03/polkadot
}

start_relaychain () {
  echo "-- RELAYCHAIN --"

  purge_relaychain_data
  
  echo "Starting relaychain..."
  docker-compose -f ./relaychain.yml --env-file ./env up -d
}

stop_relaychain () {
  docker-compose -f ./relaychain.yml --env-file env stop
  echo "RELAYCHAIN stopped"
}

start_parachain () {
  echo "-- PARACHAIN --"

  purge_parachain_data
  
  echo "Starting parachain..."
  docker-compose -f ./parachain.yml --env-file ./env up -d

  echo "Parachain started with id: $PARA_ID"
}

stop_parachain () {
  docker-compose -f ./parachain.yml --env-file env stop 
  echo "PARACHAIN stopped"
}


set_collator_keys () {
  echo "Set keys for collator-01"
  curl http://127.0.0.1:1133 -H "Content-Type:application/json;charset=utf-8" -d "@./parachain/keys/coll-01.json"
  echo "Set keys for collator-02"
  curl http://127.0.0.1:2233 -H "Content-Type:application/json;charset=utf-8" -d "@./parachain/keys/coll-02.json"
  echo "Set keys for collator-03"
  curl http://127.0.0.1:3333 -H "Content-Type:application/json;charset=utf-8" -d "@./parachain/keys/coll-03.json"
}

generate_para_state () {
  ./$PARA_BINARY export-genesis-state --chain $PARACHAIN_SPEC --parachain-id $PARA_ID > $PARA_ID.state
  ./$PARA_BINARY export-genesis-wasm --chain $PARACHAIN_SPEC > $PARA_ID.wasm

  echo -e "Created files:\n\t./$PARA_ID.state\n\t./$PARA_ID.wasm"
}

while [ "$1" != "" ]; do
  PARAM=`echo $1 | awk -F= '{print $1}'`
  VALUE=`echo $1 | awk -F= '{print $2}'`
  case $PARAM in
    -h | --help)
      usage
      exit
      ;;
    --start-para)
      start_parachain
      ;;
    --start-relay)
      start_relaychain
      ;;
    --start)
      start_relaychain
      start_parachain
      ;;
    --stop-relay)
      stop_relaychain
      ;;
    --stop-para)
      stop_parachain
      ;;
    --stop)
      stop_parachain
      stop_relaychain
      ;;
    --set-collator-keys)
      set_collator_keys
      ;;
    --get-para-state-wasm)
      generate_para_state
      ;;
    *)
      echo "ERROR: unknown parameter \"$PARAM\""
      usage
      exit 1
      ;;
  esac
  shift
done

