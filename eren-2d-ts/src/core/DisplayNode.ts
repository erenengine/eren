import { Container } from 'pixi.js';
import GameObject from './GameObject.js';

export default class DisplayNode {
  constructor(private container: Container) { }

  addTo(parent: GameObject): this {
    parent.container.addChild(this.container);
    return this;
  }

  remove() {
    this.container.destroy({ children: true });
  }

  show() {
    this.container.visible = true;
  }

  hide() {
    this.container.visible = false;
  }
}