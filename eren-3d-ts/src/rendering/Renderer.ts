import { ArcRotateCamera, Engine, HemisphericLight, MeshBuilder, Scene, StandardMaterial, Texture, Vector3 } from '@babylonjs/core';
import Stats from 'stats.js';
import GameSettings from '../GameSettings.js';
import Camera from './Camera.js';
import Light from '../lights/Light.js';

export default class Renderer {

  width: number;
  height: number;

  private canvas: HTMLCanvasElement;
  private stats: Stats | undefined;
  private scene: Scene;

  constructor(gameSize: `${number}x${number}`) {
    this.width = parseInt(gameSize.split('x')[0]);
    this.height = parseInt(gameSize.split('x')[1]);

    this.canvas = document.createElement('canvas');
    document.body.appendChild(this.canvas);

    this.resizeCanvas();
    window.addEventListener('resize', this.resizeCanvas.bind(this));

    const engine = new Engine(this.canvas, true);
    this.scene = new Scene(engine);

    const box = MeshBuilder.CreateBox("box", { size: 2 }, this.scene);

    const boxMaterial = new StandardMaterial("boxMaterial", this.scene);
    boxMaterial.diffuseTexture = new Texture("assets/sprite.jpg", this.scene);
    box.material = boxMaterial;

    this.scene.registerBeforeRender(() => {
      box.rotation.y += 0.01;
      box.rotation.x += 0.005;
    });

    if (GameSettings.debug) {
      this.stats = new Stats();
      this.stats.showPanel(0);
      document.body.appendChild(this.stats.dom);
    }

    engine.runRenderLoop(() => this._render());

    if (GameSettings.limitFPSWhenUnfocused) {
      if (!document.hasFocus()) this.changeFPS(6);
      window.addEventListener("blur", () => this.changeFPS(6));
      window.addEventListener("focus", () => this.changeFPS(undefined));
      window.addEventListener("pageshow", (event) => {
        if (event.persisted) this.changeFPS(undefined);
      });
    }
  }

  private _render = () => {
    this.scene.render();
    this.stats?.update();
  };

  private changeFPS(fps: number | undefined) {
    if (fps === undefined) {
      this._render = () => {
        this.scene.render();
        this.stats?.update();
      };
    } else {

      const frameDuration = 1000 / fps;

      let lastFrameTime = performance.now();

      this._render = () => {
        const now = performance.now();
        const delta = now - lastFrameTime;

        if (delta >= frameDuration) {
          this.scene.render();
          this.stats?.update();
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