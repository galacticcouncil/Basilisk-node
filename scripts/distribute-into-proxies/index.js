const log = console.log
console.warn = () => {}
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const { encodeAddress, cryptoWaitReady } = require('@polkadot/util-crypto')
const assert = require('assert')
const { stringToU8a } = require('@polkadot/util')
const BigNumber = require('bignumber.js')

const ACCOUNT_SECRET = process.env.ACCOUNT_SECRET || '//Alice'
const RPC = process.env.RPC_SERVER || 'ws://127.0.0.1:9988'

let proxyIndex = 2000
const multisig = 'bXiGFV6H2JpQYqXFtQtRqXNn7zS22Nco349BE7CvUzBuidraU'
const UNIT = 1000000000000

const vesting = {
  start: '13517962',
  period: '1752',
  per_period: '',
  period_count: '6000',
}

const allocation = [
  ['450000000', vesting],
  ['450000000', vesting],
  ['450000000', vesting],
  ['225000000', vesting],
  ['225000000', vesting],
  ['225000000', vesting],
  ['225000000', vesting],
  ['180000000', vesting],
  ['168750000', vesting],
  ['146250000', vesting],
  ['112500000', vesting],
  ['112500000', vesting],
  ['112500000', vesting],
  ['112500000', vesting],
  ['112500000', vesting],
  ['45000000', vesting],
  ['22500000', vesting]
]

const galactic = ['11623062500', {
  start: '13519714',
  period: '1752',
  per_period: '1937500000000000000',
  period_count: '5999',
}]

const total = allocation
  .reduce((acc, [amount]) => acc.plus(amount), new BigNumber(0))
  .multipliedBy(UNIT)
  .toFixed()
log('total to be distributed:', total)

function calculateSchedule([amount, { start, period, period_count }]) {
  const total = new BigNumber(amount).multipliedBy(UNIT)

  const per_period = total
    .div(period_count)
    .decimalPlaces(0, BigNumber.ROUND_FLOOR)
    .toFixed()
  const remainder = total.mod(period_count).toFixed()

  return {
    remainder,
    schedule: {
      start,
      period,
      per_period,
      period_count,
    },
  }
}

const distribution = allocation.map(calculateSchedule)

const totalDistributed = distribution
  .reduce(
    (acc, { schedule: { per_period, period_count }, remainder }) =>
      acc
        .plus(remainder)
        .plus(new BigNumber(per_period).multipliedBy(period_count)),
    new BigNumber(0),
  )
  .toFixed()

assert.equal(total, totalDistributed, 'total distributed does not match')
distribution.forEach(({remainder}) => assert.equal(remainder, 0, 'remainder is not zero'));

const bsxAddress = (pubKey) => encodeAddress(pubKey, 10041)
const sendAndWait = (from, tx, nonce = -1) =>
  new Promise(async (resolve, reject) => {
    try {
      await tx.signAndSend(from, { nonce }, (receipt) => {
        if (receipt.status.isInBlock) {
          resolve(receipt)
        }
      })
    } catch (e) {
      reject(e)
    }
  })

async function main() {
  await cryptoWaitReady()
  const provider = new WsProvider(RPC)
  const keyring = new Keyring({ type: 'sr25519' })
  const api = await ApiPromise.create({ provider })
  const [chain, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.version(),
  ])
  log(`connected to ${RPC} (${chain} ${nodeVersion})`)
  const from = keyring.addFromUri(ACCOUNT_SECRET)
  const activeAccount = bsxAddress(from.addressRaw)
  log('active account:', activeAccount)
  const treasuryPubKey = stringToU8a('modlpy/trsry'.padEnd(32, '\0'))
  const treasury = bsxAddress(treasuryPubKey)
  log('treasury account:', treasury)

  log('creating anonymous proxies...')
  const proxies = distribution.map(() =>
    api.tx.proxy.anonymous('Any', 0, proxyIndex++),
  )
  const receipt1 = await sendAndWait(from, api.tx.utility.batchAll(proxies))
  const anonymousProxies = receipt1.events
    .filter(({ event }) => event.method === 'AnonymousCreated')
    .map(({ event }) => event.data.anonymous.toHuman())
  assert.equal(
    anonymousProxies.length,
    distribution.length,
    'not all proxies created',
  )
  log('proxies created:', anonymousProxies)

  log('funding proxies...')
  const transfers = anonymousProxies.map((anon) =>
    api.tx.balances.forceTransfer(activeAccount, anon, 500 * UNIT),
  )
  const receipt2 = await sendAndWait(
    from,
    api.tx.sudo(api.tx.utility.batchAll(transfers)),
  )
  const transferEvents = receipt2.events.filter(
    ({ event }) => event.method === 'Transfer',
  )
  assert.equal(
    transferEvents.length,
    anonymousProxies.length,
    'not all proxies funded',
  )
  log('all proxies funded')

  log('changing delegate to multisig...')
  const changes = anonymousProxies.map((anon) =>
    api.tx.proxy.proxy(
      anon,
      null,
      api.tx.utility.batchAll([
        api.tx.proxy.removeProxy(activeAccount, 'Any', 0),
        api.tx.proxy.addProxy(multisig, 'Any', 0),
      ]),
    ),
  )
  const receipt3 = await sendAndWait(from, api.tx.utility.batchAll(changes))
  const newDelegates = receipt3.events
    .filter(({ event }) => event.method === 'ProxyAdded')
    .map(({ event }) => event.data.delegatee.toHuman())
  newDelegates.forEach((delegate) =>
    assert.equal(delegate, multisig, 'not all proxies delegated to multisig'),
  )
  log('all proxies delegated to multisig')

  log('distributing funds...')
  const toTreasury = api.tx.balances.transfer(treasury, total)
  const vestings = distribution
    .map(({ remainder, schedule }, i) => [
      api.tx.sudo.sudoAs(
        treasury,
        api.tx.vesting.vestedTransfer(anonymousProxies[i], schedule),
      ),
      //   No remainders
      //   api.tx.sudo.sudoAs(
      //     treasury,
      //     api.tx.balances.transfer(anonymousProxies[i], remainder),
      //   ),
    ])
    .flat()
  const receipt4 = await sendAndWait(
    from,
    api.tx.utility.batchAll([toTreasury, ...vestings]),
  )
  const transferred = receipt4.events
    .filter(({ event }) => event.method === 'Transfer')
    .map(({ event }) => event.data.amount.toString())
    .reduce((a, num) => a.plus(num), new BigNumber(0))
    .minus(total)
    .toFixed()
  assert.equal(transferred, total, 'difference between total and transferred')
  log('funds distributed:', transferred)
}

main()
  .then(() => {
    process.exit(0)
  })
  .catch((e) => {
    console.error(e)
    process.exit(1)
  })
