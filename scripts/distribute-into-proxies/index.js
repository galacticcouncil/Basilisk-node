const log = console.log
console.warn = () => {}
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const { encodeAddress, cryptoWaitReady } = require('@polkadot/util-crypto')
const assert = require('assert')
const { stringToU8a } = require('@polkadot/util')
const BigNumber = require('bignumber.js')
const jsonDiff = require('json-diff')

const ACCOUNT_SECRET = process.env.ACCOUNT_SECRET || '//Alice'
const RPC = process.env.RPC_SERVER || 'wss://basilisk-rpc.dwellir.com'

let proxyIndex = 2000
const multisig = 'bXiGFV6H2JpQYqXFtQtRqXNn7zS22Nco349BE7CvUzBuidraU'
const UNIT = 1000000000000

const vesting = {
  start: '13517962',
  period: '1752',
  per_period: '',
  period_count: '6000',
}


const deployedProxies = ['bXmgv5dn2GCz6TTZ5dgKMAGQH79PPxB6uKDmRrR2RLUWWmpTv',
  'bXkeb9mBW5nXNdDhVMnsuwUcAY8UC85rfB1Hds5GzRcJNUR95',
  'bXj9xqzS9Xj6vSiMEJ4XSWqarfV7zV7K1Qr2sKUSF7LbRaPG8',
  'bXhtjfkUF9M24sNM9LTpGJSooLG4gzihiMUZXwLsZGCK1sXZR',
  'bXhAHcgEpYS94STbgnJwtFKCGyNa5zjaYS1vwzGLS8icWTTrN',
  'bXioQFA5bxKsZfdKmsRuDFNR3qbSsCDGQKNLPXV1ghS6Th5e5',
  'bXkqxjGmqpvDKgzgqMjGJzNq8w2ZEvecvi3kExkaTq4kvUa8p',
  'bXkounJ7WHaShUkrsaZDU8jzTKxS24khh6nFYGSxCixc8nQ2A',
  'bXggiCArH3RSngFi5v53jWgPWAjGnYgCc5LzdrmvhrrmoQZAm',
  'bXghTpZqM6DfbjGJuSnpsbcrFJfgsAcmabMSkPFaAzjHSiSkw',
  'bXgkGQrUSgWqzE6zVxnVScZbtN45ktq3phRoKHrrbBq6SWYyr',
  'bXiUBJWy9LYhvxrBKjhQGJfJHmNS7KXSB74Zo8cWPxDZLiKmh',
  'bXjKEpwtroiytKeghbHq1xPtaUAUbAbZwNjEof4ambY9S2jDz',
  'bXkLbmD3F4wQ99Wyoy5ssoowvV1mm9poU3EBxy2RtKZAfiGjh',
  'bXmBSgpaA9r52LryffpckptGEA2q7QyjbwPhWmJSNzJCYEXmw',
  'bXmjB1dawfpTVBrpaEbQyPt1ztn4rJp1SxX1gyQZgg1S8AXXx',
  'bXmpa663XyMmgSgf2qFr3m4Fkp57W5eXsiMQqFNzcCM9upUaS',
  'bXjhPXLUcU77wg9yTeRqCaRpcKbRTQHdaxhx6r3AqAqfpWDbe'];

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
  ['22500000', vesting],
  ['11623062500', {
    start: '13519714',
    period: '1752',
    per_period: '1937500000000000000',
    period_count: '5999',
  }]
]

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
    total: total.plus(1000 * UNIT).toFixed(),
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
  const treasuryPubKey = stringToU8a('modlpy/trsry'.padEnd(32, '\0'))
  const treasury = bsxAddress(treasuryPubKey)
  log('treasury account:', treasury)



  const schedules = deployedProxies.map((proxy, i) => {
    const {schedule, total} = calculateSchedule(allocation[i]);
    return {proxy, schedules: [api.registry.createType('OrmlVestingVestingSchedule', schedule).toHuman()], balance: total};
  });


  const onchain = await Promise.all(schedules.map(async ({proxy}) => {
    const [vestingSchedules, account] = await Promise.all([
      api.query.vesting.vestingSchedules(proxy),
      api.query.system.account(proxy)
      ]);
    return {
      proxy,
      schedules: vestingSchedules.toHuman(),
      balance: new BigNumber(account.data.free).plus(account.data.reserved).toFixed()
    };
  }));

  console.log(jsonDiff.diffString(onchain, schedules, {full: true}));

  const scheduleUpdates = deployedProxies.map((proxy, i) => {
    const {schedule} = calculateSchedule(allocation[i]);
    return api.tx.vesting.updateVestingSchedules(proxy, [api.registry.createType('OrmlVestingVestingSchedule', schedule)]);
  });

  const neededUpdates = scheduleUpdates.filter((_,i) => jsonDiff.diff(schedules[i].schedules, onchain[i].schedules));
  const transferBacks = onchain.map(({proxy, balance}, i) => [proxy, new BigNumber(balance).minus(schedules[i].balance)])
    .filter(([,diff]) => diff.gt(0))
    .map(([proxy, diff]) => api.tx.balances.forceTransfer(proxy, treasury, diff.toFixed()));

  console.log('------ schedule updates')
  console.log(api.tx.utility.batchAll(neededUpdates).toHex());
  console.log('------ transfer back treasury')
  console.log(api.tx.utility.batchAll(transferBacks).toHex());

  process.exit(0)
  const from = keyring.addFromUri(ACCOUNT_SECRET)
  const activeAccount = bsxAddress(from.addressRaw)
  log('active account:', activeAccount)

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
      api.tx.balances.forceTransfer(activeAccount, anon, 1000 * UNIT),
  )
  const receipt2 = await sendAndWait(
      from,
      api.tx.sudo.sudo(api.tx.utility.batchAll(transfers)),
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
  const toTreasury = api.tx.sudo.sudo(api.tx.balances.forceTransfer(activeAccount, treasury, total))
  const vestings = distribution
      .map(({remainder, schedule}, i) =>
          api.tx.sudo.sudoAs(
              treasury,
              api.tx.vesting.vestedTransfer(anonymousProxies[i], schedule),
          ));
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
