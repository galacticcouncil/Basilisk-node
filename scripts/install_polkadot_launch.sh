npm install -g yarn
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
git checkout d696b0e04beca3368ea60f6b496722906abf0afc
yarn
yarn build
chmod +x dist/index.js
npm link
