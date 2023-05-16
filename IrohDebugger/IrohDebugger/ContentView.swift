//
//  ContentView.swift
//  IrohDebugger
//
//  Created by Brendan O'Brien on 5/15/23.
//

import SwiftUI
import Iroh

struct ContentView: View {
    @State private var cidInput: String = ""
    @State private var peerIdInput: String = ""
    @State private var allInOneInput: String = ""
    @State private var img: UIImage = UIImage()
    
    func getDocumentDirectoryPath() -> String {
        let paths = NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true)
        let documentsDirectory = paths[0]
        return documentsDirectory
    }

    var body: some View {
        VStack {
            Image(uiImage: img)
                .resizable()
                .aspectRatio(contentMode: .fit)
//            TextField("paste iroh all-in-one ticket", text: $allInOneInput)
//                            .padding()
//                            .border(Color.gray)
            TextField("cid", text: $cidInput)
                .padding()
                .border(Color.gray)
            TextField("peerID", text: $peerIdInput)
                .padding()
                .border(Color.gray)
            Button("fetch") {
                let docs = self.getDocumentDirectoryPath()
//                    get(cidInput, peerIdInput, "127.0.0.1:4433", docs)
//                get_ticket(textInput, docs)
                Iroh
                let imageURL = URL(fileURLWithPath: docs).appendingPathComponent("dragon.png")
                if let img = UIImage(contentsOfFile: imageURL.path) {
                    self.img = img
                }
            }
        }
        .padding()
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
