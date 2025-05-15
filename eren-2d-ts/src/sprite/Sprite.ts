import { Assets, Sprite as PixiSprite, Texture } from 'pixi.js';
import DisplayNode from '../core/DisplayNode';

export default class Sprite extends DisplayNode {
  constructor(x: number, y: number, assetId: string) {
    const texture = Assets.get<Texture>(assetId);
    super(new PixiSprite({ x, y, texture }));
  }
}