import { Container } from 'pixi.js';
import Renderer from '../rendering/Renderer.js';
import GameObject from './GameObject.js';

export default class DisplayNode {
  private renderer?: Renderer;
  private children: DisplayNode[] = [];

  constructor(private container: Container) { }

  get x(): number { return this.container.x; }
  set x(value: number) { this.container.x = value; }
  get y(): number { return this.container.y; }
  set y(value: number) { this.container.y = value; }

  private _layer?: string;
  get layer(): string | undefined { return this._layer; }
  set layer(value: string | undefined) {
    this._layer = value;
    this.attachToLayer();
  }

  private attachToLayer() {
    if (this.renderer && this._layer) {
      this.renderer['attachToLayer'](this, this._layer);
    }
  }

  addTo(parent: GameObject): this {
    this.renderer = parent.renderer;
    this.attachToLayer();
    parent.children.push(this);
    parent.container.addChild(this.container);
    return this;
  }

  update(deltaMS: number) {
    // Override this method to implement custom update logic
  }

  private _update(deltaMS: number) {
    this.update(deltaMS);
    for (const child of this.children) {
      child._update(deltaMS);
    }
  }

  show() { this.container.visible = true; }
  hide() { this.container.visible = false; }

  remove() {
    this.container.destroy({ children: true });
  }
}