declare module "worker-loader!*" {
  class WebpackWorker extends Worker {
    constructor();
  }

  export default WebpackWorker;
}

declare module "*.png" {
  const content: string;
  export default content;
}

// declare module "*.svg" {
//   const content: string;
//   export default content;
// }