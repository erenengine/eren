import { Button as PixiButton } from '@pixi/ui';
import { Assets, Sprite as PixiSprite, Texture } from 'pixi.js';
import DisplayNode from '../core/DisplayNode.js';

export default class Button extends DisplayNode {
  constructor(x: number, y: number, assetId: string, onPress: () => void) {
    const texture = Assets.get<Texture>(assetId);
    if (!texture) throw new Error(`Texture with id ${assetId} not found`);

    const button = new PixiButton(new PixiSprite({ texture, anchor: 0.5 }));
    button.view.position.set(x, y);
    button.onPress.connect(onPress);
    super(button.view);
  }
}