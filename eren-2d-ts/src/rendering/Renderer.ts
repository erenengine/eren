import { Application, Graphics, Rectangle, RenderLayerClass } from 'pixi.js';
import Stats from 'stats.js';
import GameObject from '../core/GameObject.js';
import GameSettings from '../GameSettings.js';
import DisplayNode from '../core/DisplayNode.js';
import { initDevtools } from '@pixi/devtools';

export default class Renderer {
  stage = new GameObject(0, 0);

  width: number;
  height: number;

  private app = new Application();
  private layers: Record<string, RenderLayerClass> = {};

  private letterboxes = {
    top: new Graphics(),
    bottom: new Graphics(),
    left: new Graphics(),
    right: new Graphics(),
  };

  constructor(gameSize: `${number}x${number}`) {
    this.width = parseInt(gameSize.split('x')[0]);
    this.height = parseInt(gameSize.split('x')[1]);
    this.init();
  }

  private async init() {
    await this.app.init({ resizeTo: window });
    document.body.appendChild(this.app.canvas);

    if (GameSettings.debug) {
      initDevtools(this.app);

      const stats = new Stats();
      stats.showPanel(0);
      document.body.appendChild(stats.dom);

      this.app.ticker.add(() => stats.update());
    }

    this.resize();
    this.app.renderer.on('resize', this.resize.bind(this));

    const letterboxLayer = new RenderLayerClass();
    letterboxLayer.zIndex = Infinity;
    letterboxLayer.sortableChildren = false;
    letterboxLayer.attach(
      this.letterboxes.top,
      this.letterboxes.bottom,
      this.letterboxes.left,
      this.letterboxes.right,
    );
    this.app.stage.addChild(letterboxLayer);

    this.app.stage.addChild(new Graphics().rect(0, 0, this.width, this.height).fill(0x304C79));

    this.stage['renderer'] = this;
    this.stage['container'].pivot.set(-this.width / 2, -this.height / 2);
    this.app.stage.addChild(this.stage['container']);

    if (GameSettings.limitFPSWhenUnfocused) {
      if (!document.hasFocus()) this.app.ticker.maxFPS = 6;
      window.addEventListener("blur", () => this.app.ticker.maxFPS = 6);
      window.addEventListener("focus", () => this.app.ticker.maxFPS = 0);
      window.addEventListener("pageshow", (event) => {
        if (event.persisted) this.app.ticker.maxFPS = 0;
      });
    }

    this.app.ticker.add((ticker) => this.stage['_update'](ticker.deltaMS));
  }

  private resize() {
    const scaleX = this.app.renderer.width / this.width;
    const scaleY = this.app.renderer.height / this.height;
    const scale = Math.min(scaleX, scaleY);

    this.app.stage.scale = scale;
    const viewportW = this.width * scale;
    const viewportH = this.height * scale;
    const padX = (this.app.renderer.width - viewportW) / 2;
    const padY = (this.app.renderer.height - viewportH) / 2;
    this.app.stage.position.set(padX, padY);

    const { top, bottom, left, right } = this.letterboxes;
    top.clear().rect(0, -padY / scale, this.width / scale, padY / scale).fill(0x000000);
    bottom.clear().rect(0, this.height, this.width / scale, padY / scale).fill(0x000000);
    left.clear().rect(-padX / scale, 0, padX / scale, this.height / scale).fill(0x000000);
    right.clear().rect(this.width, 0, padX / scale, this.height / scale).fill(0x000000);
  }

  private attachToLayer(displayNode: DisplayNode, layer: string) {
    if (!this.layers[layer]) {
      this.layers[layer] = new RenderLayerClass();
      this.app.stage.addChild(this.layers[layer]);
    }
    this.layers[layer].attach(displayNode['container']);
  }
}