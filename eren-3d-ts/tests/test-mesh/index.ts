import AssetLoader from '../../dist/assets/AssetLoader.js';
import GameSettings from '../../dist/GameSettings.js';
import Camera from '../../dist/rendering/Camera.js';
import Renderer from '../../dist/rendering/Renderer.js';

GameSettings.debug = true;
GameSettings.limitFPSWhenUnfocused = true;

const renderer = new Renderer('1280x720');
renderer.camera = new Camera(
  5 * Math.sin(Math.PI / 4) * Math.cos(Math.PI / 4),
  5 * Math.cos(Math.PI / 4),
  5 * Math.sin(Math.PI / 4) * Math.sin(Math.PI / 4)
);

await AssetLoader.load([
  { id: 'character-female-a', src: 'assets/kenney-mini-characters/character-female-a.glb' },
]);
