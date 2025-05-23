import { Assets, AnimatedSprite as PixiAnimatedSprite, Texture } from 'pixi.js';
import DisplayNode from '../core/DisplayNode.js';

export default class AnimatedSprite extends DisplayNode {
  constructor(x: number, y: number, assetId: string, options: {
    fps: number;
    loop?: boolean;
  }) {
    const spritesheet = Assets.get(assetId);
    if (!spritesheet) throw new Error(`Spritesheet with id ${assetId} not found`);

    const sprite = new PixiAnimatedSprite(spritesheet.animations['default']);
    sprite.position.set(x, y);
    sprite.anchor = 0.5;
    sprite.animationSpeed = options.fps / 60;
    sprite.loop = options.loop ?? true;
    sprite.play();
    super(sprite);
  }
}