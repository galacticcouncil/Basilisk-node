npm install -g yarn
npm install -g typescript
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
git checkout 6f66f9f0b75fafbf650a79f9aa5d3fdbda28dcda
yarn
yarn build
chmod +x dist/index.js
npm link
