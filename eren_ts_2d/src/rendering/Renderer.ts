import { FPSDisplay, GameSettings } from '@erenengine/core';
import { initDevtools } from '@pixi/devtools';
import { Application, Graphics, RenderLayerClass } from 'pixi.js';
import DisplayNode from '../core/DisplayNode.js';
import GameObject from '../core/GameObject.js';

export default class Renderer {
  stage = new GameObject(0, 0);

  private app = new Application();
  private layers: Record<string, RenderLayerClass> = {};
  private letterbox = new Graphics();

  constructor(public width: number, public height: number) {
    this.init();
  }

  private async init() {
    await this.app.init({ resizeTo: window, resolution: window.devicePixelRatio, autoDensity: true });
    document.body.appendChild(this.app.canvas);

    if (GameSettings.debug) {
      initDevtools(this.app);

      const fpsDisplay = new FPSDisplay();
      this.app.ticker.add(() => fpsDisplay.update());
    }

    this.resize();
    this.app.renderer.on('resize', this.resize.bind(this));

    const letterboxLayer = new RenderLayerClass();
    letterboxLayer.zIndex = Infinity;
    letterboxLayer.sortableChildren = false;
    letterboxLayer.attach(this.letterbox);
    this.app.stage.addChild(letterboxLayer);

    this.app.stage.addChild(new Graphics().rect(0, 0, this.width, this.height).fill(0x304C79));

    this.stage['renderer'] = this;
    this.stage['container'].pivot.set(-this.width / 2, -this.height / 2);
    this.app.stage.addChild(this.stage['container']);

    if (GameSettings.limitFPSWhenUnfocused) {
      if (!document.hasFocus()) this.app.ticker.maxFPS = 6;
      window.addEventListener('blur', () => this.app.ticker.maxFPS = 6);
      window.addEventListener('focus', () => this.app.ticker.maxFPS = 0);
      window.addEventListener('pageshow', (event) => {
        if (event.persisted) this.app.ticker.maxFPS = 0;
      });
    }

    this.app.ticker.add((ticker) => this.stage['_update'](ticker.deltaMS));
  }

  private resize() {
    const { renderer, stage } = this.app;

    const scaleX = renderer.width / this.width;
    const scaleY = renderer.height / this.height;
    const scale = Math.min(scaleX, scaleY);
    stage.scale = scale;

    const viewportW = this.width * scale;
    const viewportH = this.height * scale;
    const padX = (renderer.width - viewportW) / 2;
    const padY = (renderer.height - viewportH) / 2;
    stage.position.set(padX, padY);

    this.letterbox.clear()
      .rect(0, -padY / scale, this.width / scale, padY / scale)
      .rect(0, this.height, this.width / scale, padY / scale)
      .rect(-padX / scale, 0, padX / scale, this.height / scale)
      .rect(this.width, 0, padX / scale, this.height / scale).fill(0x000000);
  }

  private attachToLayer(displayNode: DisplayNode, layer: string) {
    if (!this.layers[layer]) {
      this.layers[layer] = new RenderLayerClass();
      this.app.stage.addChild(this.layers[layer]);
    }
    this.layers[layer].attach(displayNode['container']);
  }
}