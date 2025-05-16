import { Application, Graphics } from 'pixi.js';
import GameObject from '../core/GameObject.js';

export default class Renderer {
  stage = new GameObject(0, 0);

  private app = new Application();
  private width = 0;
  private height = 0;

  constructor(gameSize: `${number}x${number}`) {
    this.width = parseInt(gameSize.split('x')[0]);
    this.height = parseInt(gameSize.split('x')[1]);
    this.init();
  }

  private async init() {
    await this.app.init({ resizeTo: window });
    document.body.appendChild(this.app.canvas);

    this.resize();
    this.app.renderer.on('resize', this.resize.bind(this));

    /*const graphics = new Graphics().rect(this.width / 2, this.height / 2, this.width, this.height).fill(0xFF0000);
    graphics.pivot.set(this.width / 2, this.height / 2);
    this.app.stage.addChild(graphics);*/
  }

  private resize() {
    const scaleX = this.app.renderer.width / this.width;
    const scaleY = this.app.renderer.height / this.height;
    const scale = Math.min(scaleX, scaleY);
    this.app.stage.scale = scale;

    this.app.stage.position.set(
      (this.app.renderer.width - this.width * scale) / 2,
      (this.app.renderer.height - this.height * scale) / 2,
    );
  }
}