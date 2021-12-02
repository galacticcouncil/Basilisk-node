const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {encodeAddress, cryptoWaitReady} = require("@polkadot/util-crypto");
const {bufferToU8a, u8aToHex} = require("@polkadot/util");
const assert = require("assert");
const fs = require('fs');

const ACCOUNT_SECRET = process.env.ACCOUNT_SECRET || "//Alice";

let api;
const filename = process.argv[2];
const code = u8aToHex(bufferToU8a(fs.readFileSync(filename)));
let header = null;

const hdxAddress = (pubKey) => encodeAddress(pubKey, 63);

async function main() {
  await cryptoWaitReady();
  const provider = new WsProvider('ws://127.0.0.1:9988');
  const keyring = new Keyring({type: "sr25519"});
  const from = keyring.addFromUri(ACCOUNT_SECRET);
  api = await ApiPromise.create({provider});

  const getSpecVersion = () => Number(api.runtimeVersion.specVersion.toString());

  const waitForBlock = async number => new Promise(resolve => {
    let count = 0;
    api.rpc.chain.subscribeNewHeads(async header => {
      console.log(`block #${header.number}`);
      if (++count === number) resolve();
    });
  });

  const waitTx = async tx => new Promise(resolve =>
    tx.signAndSend(from, ({status}) => {
      if (status.isInBlock) {
        console.log("tx Included");
      } else {
        console.log("tx " + status.type);
      }
      if (status.type === "Finalized") {
        resolve();
      }
    }));

  const watchEventsFromSections = sections => api.query.system.events(events => events
    .filter(({event: {section}}) =>
      sections.includes(section)
    )
    .forEach(({event: {data, method, section}}) =>
      console.log(`event ${section}.${method} ${data.toString()}`)
    ));

  const onEvent = (event, callback) => api.query.system.events(events => events
    .filter(({event: {section, method}}) => event === `${section}.${method}`)
    .forEach(callback));

  const [chain, nodeVersion, sudoKey] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.version(),
    api.query.sudo.key()
  ]);
  const specVersion = getSpecVersion();

  console.log(`connected to ${chain} ${nodeVersion}`);
  console.log(`current runtime version ${specVersion}`);

  assert.equal(hdxAddress(sudoKey), hdxAddress(from.addressRaw), `imported account doesn't match sudo key`);
  const setCode = api.tx.system.setCode(code);
  const sudo = api.tx.sudo.sudoUncheckedWeight(setCode, 100);

  console.log('waiting for parachain to start producing blocks');
  await waitForBlock(3);

  console.log('submitting runtime upgrade');
  watchEventsFromSections(["sudo", "parachainSystem"]);
  await waitTx(sudo);

  await new Promise(resolve => onEvent('parachainSystem.ValidationFunctionApplied', resolve));
  api.rpc.chain.subscribeNewHeads(async header => {
    const newSpec = getSpecVersion();
    if (specVersion < newSpec) {
      console.log(`parachain was sucessfully upgraded ${specVersion} -> ${newSpec}`);
      process.exit(0);
    } else {
      console.log(`api still on the older spec (${newSpec})`);
    }
  });
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});

setTimeout(() => {
  if (header == null) {
    console.log(`upgrade was not performed within 10 minutes`);
    process.exit(1);
  }
}, 10 * 60000)

