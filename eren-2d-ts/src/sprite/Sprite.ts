import { Assets, Sprite as PixiSprite, Texture } from 'pixi.js';
import GameNode from '../core/GameNode';

export default class Sprite extends GameNode {
  constructor(x: number, y: number, assetId: string) {
    const texture = Assets.get<Texture>(assetId);
    super(new PixiSprite({ x, y, texture }));
  }
}