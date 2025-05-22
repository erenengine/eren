import { AbstractMesh, AnimationGroup, AssetsManager } from '@babylonjs/core';
import '@babylonjs/loaders';

type ProgressCallback = (progress: number) => void;

class AssetLoader {
  private models: {
    [id: string]: {
      meshes: AbstractMesh[];
      animationGroups: { [animationName: string]: AnimationGroup; };
    };
  } = {};

  public getModelData(id: string) {
    if (this.models[id]) {
      return this.models[id];
    }
    throw new Error(`Model with id ${id} not found`);
  }

  async load(
    assets: { id: string; src: string; }[],
    onProgress?: ProgressCallback,
  ) {
    return new Promise<void>((resolve, reject) => {
      const assetsManager = new AssetsManager();
      for (const asset of assets) {
        const splitSrc = asset.src.split("/");
        const rootUrl = splitSrc.slice(0, splitSrc.length - 1).join("/") + "/";
        const fileName = splitSrc[splitSrc.length - 1];

        if (fileName.endsWith(".glb")) {
          const task = assetsManager.addMeshTask(asset.id, "", rootUrl, fileName);
          task.onSuccess = (task) => {
            console.log(`Loaded ${task.name}`);

            const animationGroups: { [animationName: string]: AnimationGroup; } = {};
            for (const animationGroup of task.loadedAnimationGroups) {
              const animationName = animationGroup.name;
              animationGroups[animationName] = animationGroup;
            }

            this.models[asset.id] = {
              meshes: task.loadedMeshes,
              animationGroups
            };
          };
          task.onError = (task) => {
            console.error(`Failed to load ${task.name}: ${task.errorObject.message}`);
            reject(new Error(`Failed to load ${task.name}: ${task.errorObject.message}`));
          };
        }

        else {
          throw new Error(`Unsupported asset type: ${fileName}`);
        }
      }
      assetsManager.onProgress = (remainingCount, totalCount, lastFinishedTask) => {
        const progress = (totalCount - remainingCount) / totalCount;
        if (onProgress) {
          onProgress(progress);
        }
        console.log(`Loading: ${lastFinishedTask.name} (${remainingCount}/${totalCount})`);
      };
      assetsManager.onFinish = () => {
        console.log("All assets loaded", this.models);
        resolve();
      };
      assetsManager.load();
    });
  }
}

export default new AssetLoader();
