import Stats from 'stats.js';

export default class FPSDisplay {
  private stats: Stats;

  constructor() {
    this.stats = new Stats();
    this.stats.showPanel(0);
    document.body.appendChild(this.stats.dom);
  }

  public update() {
    this.stats.update();
  }
}