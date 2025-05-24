import { Engine, Scene, TransformNode } from '@babylonjs/core';
import { FPSDisplay, GameSettings } from '@erenengine/core';
import DisplayNode from '../core/DisplayNode.js';
import Light from '../lights/Light.js';
import Camera from './Camera.js';

export default class Renderer {
  stage: DisplayNode;

  private canvas: HTMLCanvasElement;
  private scene: Scene;
  private fpsDisplay: FPSDisplay | undefined;

  constructor(public width: number, public height: number) {
    this.canvas = document.createElement('canvas');
    document.body.appendChild(this.canvas);

    this.resizeCanvas();
    window.addEventListener('resize', this.resizeCanvas.bind(this));

    const engine = new Engine(this.canvas, true);
    this.scene = new Scene(engine);
    this.stage = new DisplayNode(new TransformNode('', this.scene));

    if (GameSettings.debug) this.fpsDisplay = new FPSDisplay();

    engine.runRenderLoop(() => this._render());

    if (GameSettings.limitFPSWhenUnfocused) {
      if (!document.hasFocus()) this.changeFPS(6);
      window.addEventListener('blur', () => this.changeFPS(6));
      window.addEventListener('focus', () => this.changeFPS(undefined));
      window.addEventListener('pageshow', (event) => {
        if (event.persisted) this.changeFPS(undefined);
      });
    }
  }

  private _render = () => {
    this.scene.render();
    this.fpsDisplay?.update();
  };

  private changeFPS(fps: number | undefined) {
    if (fps === undefined) {
      this._render = () => {
        this.scene.render();
        this.fpsDisplay?.update();
      };
    } else {

      const frameDuration = 1000 / fps;

      let lastFrameTime = performance.now();

      this._render = () => {
        const now = performance.now();
        const delta = now - lastFrameTime;

        if (delta >= frameDuration) {
          this.scene.render();
          this.fpsDisplay?.update();
          lastFrameTime = now - (delta % frameDuration);
        }
      };
    }
  }

  private resizeCanvas() {
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;
    this.canvas.width = windowWidth;
    this.canvas.height = windowHeight;
  }

  public set camera(camera: Camera) {
    camera.setScene(this.scene);
  }

  public set light(light: Light) {
    light.setScene(this.scene);
  }
}