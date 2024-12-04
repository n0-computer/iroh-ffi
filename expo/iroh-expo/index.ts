// Import the native module. On web, it will be resolved to IrohExpo.web.ts
// and on native platforms to IrohExpo.ts
import IrohExpoModule from './src/IrohExpoModule';
import { AddrInfoOptions, ChangeEventPayload, ShareMode } from './src/IrohExpo.types';

export async function nodeId(): Promise<string> {
  return IrohExpoModule.nodeId();
}

export async function docCreate(): Promise<Doc> {
  let id = await IrohExpoModule.docCreate();
  return new Doc(id);
}

export async function docDrop(id: string) {
  return await IrohExpoModule.docDrop(id);
}

export async function docJoin(ticket: string): Promise<Doc> {
  let id = await IrohExpoModule.docJoin(ticket);
  return new Doc(id);
}


export class Doc {
  public id: string;
  
  constructor(id: string) {
    this.id = id;
  }

  async share(mode: ShareMode, addrOptions: AddrInfoOptions) {
    return IrohExpoModule.docShare(this.id, mode, addrOptions);
  }
}
