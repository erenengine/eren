import GameSettings from '../../dist/GameSettings.js';
import Renderer from '../../dist/rendering/Renderer.js';
import AssetLoader from '../../dist/assets/AssetLoader.js';
import Sprite from '../../dist/sprite/Sprite.js';

GameSettings.debug = true;
GameSettings.limitFPSWhenUnfocused = true;

const renderer = new Renderer('1280x720');

await AssetLoader.load([
  { id: 'sprite', src: 'assets/sprite.jpg' },
]);

new Sprite(0, 0, 'sprite').addTo(renderer.stage);
