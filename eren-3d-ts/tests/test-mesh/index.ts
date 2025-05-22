import AssetLoader from '../../dist/assets/AssetLoader.js';
import GameSettings from '../../dist/GameSettings.js';
import Renderer from '../../dist/rendering/Renderer.js';

GameSettings.debug = true;
GameSettings.limitFPSWhenUnfocused = true;

const renderer = new Renderer('1280x720');

await AssetLoader.load([
  { id: 'character-female-a', src: 'assets/kenney-mini-characters/character-female-a.glb' },
]);