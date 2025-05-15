import { Assets, AssetsBundle, ProgressCallback } from 'pixi.js';
import { v4 as uuidv4 } from 'uuid';

class AssetLoader {
  async load(assets: AssetsBundle['assets'], onProgress?: ProgressCallback) {
    const bundleId = uuidv4();
    Assets.addBundle(bundleId, assets);
    await Assets.loadBundle(bundleId, onProgress);
    return async () => await Assets.unloadBundle(bundleId);
  }
}

export default new AssetLoader();
