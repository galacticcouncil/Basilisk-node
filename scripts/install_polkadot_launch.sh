npm install -g yarn
git clone https://github.com/paritytech/polkadot-launch.git --depth 1
cd polkadot-launch
yarn
yarn build
chmod +x dist/index.js
npm link
