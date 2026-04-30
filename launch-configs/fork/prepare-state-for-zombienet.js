const fs = require('fs');
const {TypeRegistry} = require('@polkadot/types');
const {hexToU8a, u8aToHex} = require('@polkadot/util');
const { xxhashAsHex } = require('@polkadot/util-crypto');

// Define network names
const NEW_NAME = process.env.CHAIN_NAME || "Basilisk Local Testnet";
const NEW_ID = process.env.CHAIN_ID || "basilisk_local_testnet";
const NEW_RELAY_CHAIN = "rococo_local_testnet";

// Replacement values
// Vec<AccountId> of [//Alice, //Bob] standard sr25519 keys (what zombienet inserts into the keystore for `name: "alice"`/`name: "bob"`).
const AURA_AUTHORITIES_VALUE = "0x08d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
const TECHNICAL_COMMITTEE_VALUE = "0x04d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const SYSTEM_ACCOUNT_VALUE = "0x00000000000000000100000000000000ba31bc09df123864f700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

async function updateChainSpec(inputFile, outputFile) {
    if (fs.existsSync(outputFile)) {
        console.log(`Output file ${outputFile} already exists, skipping processing...`);
        return;
    }

    console.log('Starting the chain spec update script...');
    let chainSpec;
    try {
        chainSpec = JSON.parse(fs.readFileSync(inputFile, 'utf8'));
    } catch (err) {
        console.error('Error reading the chain spec file:', err);
        process.exit(1);
    }

    const registry = new TypeRegistry();
    registry.register({
        HydraDxMathRatio: {
            n: 'u128',
            d: 'u128',
        },
        HydradxTraitsOracleVolume: {
            aIn: 'u128',
            bOut: 'u128',
            aOut: 'u128',
            bIn: 'u128',
        },
        HydradxTraitsOracleLiquidity: {
            a: 'u128',
            b: 'u128',
        },
        EmaOracleEntry: {
            price: 'HydraDxMathRatio',
            volume: 'HydradxTraitsOracleVolume',
            liquidity: 'HydradxTraitsOracleLiquidity',
            sharesIssuance: 'Option<u128>',
            updatedAt: 'u32',
        },
        OracleValue: '(EmaOracleEntry, u32)',
        PalletLiquidityMiningFarmState: {
            _enum: ['Active', 'Terminated']
        },
        Perquintill: 'u64',
        FixedU128: 'u128',
        AccountId32: '[u8; 32]',
        PalletLiquidityMiningGlobalFarmData: {
            id: 'u32',
            owner: 'AccountId32',
            updatedAt: 'u32',
            totalSharesZ: 'u128',
            accumulatedRpz: 'FixedU128',
            rewardCurrency: 'u32',
            pendingRewards: 'u128',
            accumulatedPaidRewards: 'u128',
            yieldPerPeriod: 'Perquintill',
            plannedYieldingPeriods: 'u32',
            blocksPerPeriod: 'u32',
            incentivizedAsset: 'u32',
            maxRewardPerPeriod: 'u128',
            minDeposit: 'u128',
            liveYieldFarmsCount: 'u32',
            totalYieldFarmsCount: 'u32',
            priceAdjustment: 'FixedU128',
            state: 'PalletLiquidityMiningFarmState'
        },
        PalletLiquidityMiningLoyaltyCurve: {
            initialRewardPercentage: 'FixedU128',
            scaleCoef: 'u32',
        },
        PalletLiquidityMiningYieldFarmData: {
            id: 'u32',
            updatedAt: 'u32',
            totalShares: 'u128',
            totalValuedShares: 'u128',
            accumulatedRpvs: 'FixedU128',
            accumulatedRpz: 'FixedU128',
            loyaltyCurve: 'Option<PalletLiquidityMiningLoyaltyCurve>',
            multiplier: 'FixedU128',
            state: 'PalletLiquidityMiningFarmState',
            entriesCount: 'u64',
            leftToDistribute: 'u128',
            totalStopped: 'u32',
        },
    });

    const governance = process.env.KEEP_GOVERNANCE ? {} : {
        // Basilisk has no Council pallet (only TechnicalCommittee + OpenGov)
        "0xed25f63942de25ac5253ba64b5eb64d1ba7fb8745735dc3be2a2c61a72c39e78": TECHNICAL_COMMITTEE_VALUE, // TechnicalCommittee.members
    }

    const REPLACEMENTS = {
        "0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f": "0x2a080000", // parachainInfo.parachainId = 2090
        "0x57f8dc2f5ab09467896f47300f0424385e0621c4869aa60c02be9adcc98a0d1d": AURA_AUTHORITIES_VALUE, // aura.authorities
        "0x3c311d57d4daf52904616cf69648081e5e0621c4869aa60c02be9adcc98a0d1d": AURA_AUTHORITIES_VALUE, // auraExt.authorities
        "0xcec5070d609dd3497f72bde07fc96ba088dcde934c658227ee1dfafcd6e16903": AURA_AUTHORITIES_VALUE, // Session validators
        "0x15464cac3378d46f113cd5b7a4d71c845579297f4dfb9609e7e4c2ebab9ce40a": AURA_AUTHORITIES_VALUE, // CollatorSelection.invulnerables
        ...governance,
        "0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d": SYSTEM_ACCOUNT_VALUE, // System.Account
    };

    // Set Parameters.IsTestnet to 1
    const IS_TESTNET_KEY =
        xxhashAsHex('Parameters', 128).replace('0x', '') +
        xxhashAsHex('IsTestnet', 128).replace('0x', '');
    REPLACEMENTS[`0x${IS_TESTNET_KEY}`] = '0x01';

    // Keys to delete — drop fields that must be reinitialized for a fresh local relay/parachain
    const KEYS_TO_DELETE = [
        "0x26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac", // System.Number
        "0x26aa394eea5630e07c48ae0c9558cef799e354094e5f3f9eddda2206fb22e261", // System.ParentHash
        "0x45323df7cc47150b3930e2666b0aa313911a5dd3f1155f5b7d0c5aa102a757f9", // ParachainSystem.lastDmqMqcHead
        "0x45323df7cc47150b3930e2666b0aa3133dca42deb008c6559ee789c9b9f70a2c", // ParachainSystem.lastHrmpMqcHeads
        "0x45323df7cc47150b3930e2666b0aa313a2bca190d36bd834cc73a38fc213ecbd", // ParachainSystem.lastRelayChainBlockNumber
        "0xcec5070d609dd3497f72bde07fc96ba0e0cdd062e6eaf24295ad4ccfc41d4609", // Session.queuedKeys
        "0xcec5070d609dd3497f72bde07fc96ba072763800a36a99fdfc7c10f6415f6ee6", // Session.currentIndex
    ];

    const PREFIXES_TO_DELETE = [
        "0x5258a12472693b34a3ed25509781e55f3ffefddfbe00a43e565ba6114d1589ea", // emaOracle.accumulator
        "0xcec5070d609dd3497f72bde07fc96ba04c014e6bf8b8c2c011e7290b85696bb3", // Session.nextKeys
    ];

    KEYS_TO_DELETE.forEach((key) => delete chainSpec.genesis.raw.top[key]);

    for (const prefix of PREFIXES_TO_DELETE) {
        for (const key of Object.keys(chainSpec.genesis.raw.top)) {
            if (key.startsWith(prefix)) {
                delete chainSpec.genesis.raw.top[key];
            }
        }
    }

    // Reset EmaOracle entries' updatedAt to 0 so the oracle restarts from block 0
    console.log('Processing EmaOracleEntry & GlobalFarm updates...');
    for (const [key, value] of Object.entries(chainSpec.genesis.raw.top)) {
        if (key.startsWith("0x5258a12472693b34a3ed25509781e55fb79")) {
            try {
                const originalBytes = hexToU8a(value);
                const decoded = registry.createType('(EmaOracleEntry, u32)', originalBytes);
                const [entry, _blockNumber] = decoded;

                console.log(`🔍 Key: ${key}`);

                entry.set('updatedAt', registry.createType('u32', 0));

                const updated = registry.createType('(EmaOracleEntry, u32)', [entry, 0]);
                const reEncodedBytes = updated.toU8a();

                chainSpec.genesis.raw.top[key] = u8aToHex(reEncodedBytes);

                console.log(`✅ Updated ${key} → blockNumber reset to 0`);
            } catch (err) {
                console.error(`❌ Error processing oracle key ${key}:`, err);
            }
        } else if (key.startsWith("0xa1a851f6ddab88c23c6615f42a0062df8d84255c07d18453a739a171ac5cf629") || key.startsWith("0xae438efb85a5af0e340133650eccd7638d84255c07d18453a739a171ac5cf629")) {
            // XYK warehouse LM global farms
            try {
                const decoded = registry.createType('PalletLiquidityMiningGlobalFarmData', hexToU8a(value));
                const json = decoded.toJSON();
                json.updatedAt = 0;
                const updated = registry.createType('PalletLiquidityMiningGlobalFarmData', json);
                chainSpec.genesis.raw.top[key] = u8aToHex(updated.toU8a());
            } catch (err) {
                console.error(`Error processing globalFarm for key ${key}:`, err);
            }
        } else if (key.startsWith("0xa1a851f6ddab88c23c6615f42a0062df7e1045c712fe23a3e89096e70b7ea444") || key.startsWith("0xae438efb85a5af0e340133650eccd7637e1045c712fe23a3e89096e70b7ea444")) {
            // XYK warehouse LM yield farms
            try {
                const decoded = registry.createType('PalletLiquidityMiningYieldFarmData', hexToU8a(value));
                const json = decoded.toJSON();
                json.updatedAt = 0;
                const updated = registry.createType('PalletLiquidityMiningYieldFarmData', json);
                chainSpec.genesis.raw.top[key] = u8aToHex(updated.toU8a());
            } catch (err) {
                console.error(`Error processing yeildFarm for key ${key}:`, err);
            }
        }
    }

    for (const [key, value] of Object.entries(REPLACEMENTS)) {
        chainSpec.genesis.raw.top[key] = value;
    }

    // Optional: preauthorize a runtime upgrade so the fork comes up with
    // System.AuthorizedUpgrade populated. A single system.applyAuthorizedUpgrade(code)
    // call is then enough to enact the new wasm — no governance step required.
    if (process.env.AUTHORIZE_UPGRADE_CODE_HASH) {
        const raw = process.env.AUTHORIZE_UPGRADE_CODE_HASH.toLowerCase().replace(/^0x/, '');
        if (!/^[0-9a-f]{64}$/.test(raw)) {
            throw new Error(`AUTHORIZE_UPGRADE_CODE_HASH must be a 0x-prefixed 32-byte hex string, got: ${process.env.AUTHORIZE_UPGRADE_CODE_HASH}`);
        }
        const checkVersion = process.env.AUTHORIZE_UPGRADE_CHECK_VERSION === 'false' ? '00' : '01';
        const key = '0x' +
            xxhashAsHex('System', 128).replace('0x', '') +
            xxhashAsHex('AuthorizedUpgrade', 128).replace('0x', '');
        chainSpec.genesis.raw.top[key] = '0x' + raw + checkVersion;
        console.log(`✅ Preauthorized runtime upgrade: code_hash=0x${raw} check_version=${checkVersion === '01'}`);
    }

    // Update metadata fields
    chainSpec.name = NEW_NAME;
    chainSpec.id = NEW_ID;
    chainSpec.relay_chain = NEW_RELAY_CHAIN;
    chainSpec.para_id = 2090;

    try {
        fs.writeFileSync(outputFile, JSON.stringify(chainSpec));
        console.log(`Chain spec updated successfully and saved to ${outputFile}`);
    } catch (err) {
        console.error('Error writing the updated chain spec file:', err);
    }

    console.log('Chain spec update script completed.');
}

const inputFile = process.argv[2];
const outputFile = process.argv[3];

if (!inputFile || !outputFile) {
    console.error('Usage: node updateChainSpec.js <inputFile> <outputFile>');
    process.exit(1);
}

updateChainSpec(inputFile, outputFile).catch((error) => {
    console.error('Error updating chain spec:', error);
    process.exit(1);
});
