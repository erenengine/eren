import AssetLoader from '../../dist/assets/AssetLoader.js';
import GameSettings from '../../dist/GameSettings.js';
import Camera from '../../dist/rendering/Camera.js';
import Renderer from '../../dist/rendering/Renderer.js';
import HemisphereLight from '../../dist/lights/HemisphereLight.js';
import Model from '../../dist/model/Model.js';

GameSettings.debug = true;
GameSettings.limitFPSWhenUnfocused = true;

const renderer = new Renderer('1280x720');
renderer.camera = new Camera(0, 0, 4);
renderer.light = new HemisphereLight(1, 1, 0);

await AssetLoader.load([
  { id: 'character-female-a', src: 'assets/kenney-mini-characters/character-female-a.glb' },
]);

new Model(0, -0.3, 0, 'character-female-a').addTo(renderer.stage);
