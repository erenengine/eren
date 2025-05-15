import { Container } from 'pixi.js';
import GameObject from './GameObject';

export default class DisplayNode {
  constructor(container: Container) { }

  addTo(parent: GameObject): this {
    throw new Error('Not implemented');
    return this;
  }

  remove() {
    throw new Error('Not implemented');
  }

  show() {
    throw new Error('Not implemented');
  }

  hide() {
    throw new Error('Not implemented');
  }
}