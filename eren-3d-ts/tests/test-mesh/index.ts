import GameSettings from '../../dist/GameSettings.js';
import Renderer from '../../dist/rendering/Renderer.js';

GameSettings.debug = true;
GameSettings.limitFPSWhenUnfocused = true;

const renderer = new Renderer('1280x720');
