import { Container } from 'pixi.js';
import DisplayNode from './DisplayNode.js';

export default class GameObject extends DisplayNode {
  constructor(x: number, y: number) {
    super(new Container({ x, y }));
  }

  add(...children: DisplayNode[]): void {
    for (const child of children) {
      child.addTo(this);
    }
  }
}