import { Assets, Sprite as PixiSprite, Texture } from 'pixi.js';
import DisplayNode from '../core/DisplayNode.js';

export default class Sprite extends DisplayNode {
  constructor(x: number, y: number, assetId: string) {
    const texture = Assets.get<Texture>(assetId);
    if (!texture) throw new Error(`Texture with id ${assetId} not found`);
    super(new PixiSprite({ x, y, texture, anchor: 0.5 }));
  }
}