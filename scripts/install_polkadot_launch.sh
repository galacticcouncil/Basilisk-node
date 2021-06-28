npm install -g yarn
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
git checkout 381cc9108dd32e5260ef3b85ea49e00cda35c398
yarn
yarn build
chmod +x dist/index.js
npm link
