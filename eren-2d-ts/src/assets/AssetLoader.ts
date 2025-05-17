import { Assets, ProgressCallback, Spritesheet, SpritesheetData } from 'pixi.js';
import { v4 as uuidv4 } from 'uuid';

Assets.loader.parsers.push({
  name: 'loadSpritesheet',
  async load(src: string, options: any) {
    const texture = await Assets.load(src);
    const frameWidth = options.data.frameWidth;
    const frameHeight = options.data.frameHeight;
    const frames: Record<string, { frame: { x: number; y: number; w: number; h: number; }; }> = {};
    const frameCount = (texture.width / frameWidth) * (texture.height / frameHeight);
    for (let i = 0; i < frameCount; i++) {
      const x = (i % (texture.width / frameWidth)) * frameWidth;
      const y = Math.floor(i / (texture.width / frameWidth)) * frameHeight;
      frames[`frame_${i}`] = { frame: { x, y, w: frameWidth, h: frameHeight } };
    }
    const spritesheetData: SpritesheetData = {
      frames,
      meta: { scale: 1 },
      animations: { default: Object.keys(frames) },
    };
    const sheet = new Spritesheet(texture, spritesheetData);
    await sheet.parse();
    return sheet;
  },
});

class AssetLoader {
  async load(
    assets: { id: string; src: string; frameWidth?: number, frameHeight?: number; }[],
    onProgress?: ProgressCallback,
  ) {
    const bundleId = uuidv4();
    Assets.addBundle(bundleId, assets.map(asset => {
      if (asset.frameWidth !== undefined && asset.frameHeight !== undefined) {
        return {
          alias: asset.id,
          src: asset.src,
          data: { frameWidth: asset.frameWidth, frameHeight: asset.frameHeight },
          loadParser: 'loadSpritesheet',
        };
      } else {
        return {
          alias: asset.id,
          src: asset.src,
        };
      }
    }));
    await Assets.loadBundle(bundleId, onProgress);
    return async () => await Assets.unloadBundle(bundleId);
  }
}

export default new AssetLoader();
