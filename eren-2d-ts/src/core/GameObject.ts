import { Container } from 'pixi.js';
import GameNode from './GameNode';

export default class GameObject extends GameNode {
  constructor(x: number, y: number) {
    super(new Container({ x, y }));
  }

  add(...children: GameObject[]): void {
    throw new Error('Not implemented');
  }
}