//
//  AppDelegate.swift
//  Levels
//
//  Created by Praveen Perera on 12/22/22.
//  Copyright © 2021 Praveen Perera. All rights reserved.
//

import Cocoa
import SwiftUI
import AVFoundation

class DecibelResponderImpl: DecibelResponder {
    var statusBar: StatusBarController?
    
    func setStatusBar(sb: StatusBarController?) {
        statusBar = sb
    }

    func decibel(decibel: Int32) {
        DispatchQueue.main.async {
            self.statusBar?.changeText(text: String(decibel))
        }
    }
}

@main
class AppDelegate: NSObject, NSApplicationDelegate {

    var popover = NSPopover.init()
    var statusBar: StatusBarController?
    let levels = Levels()

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Create the SwiftUI view that provides the contents
        let contentView = ContentView()

        // Set the SwiftUI's ContentView to the Popover's ContentViewController
        popover.contentViewController = MainViewController()
        popover.contentSize = NSSize(width: 360, height: 360)
        popover.contentViewController?.view = NSHostingView(rootView: contentView)
        
        
        let cbObject = DecibelResponderImpl()
        
        
        // Create the Status Bar Item with the Popover
        statusBar = StatusBarController.init(popover)
        statusBar?.changeText(text: "h2")
        
        cbObject.setStatusBar(sb: statusBar)
        levels.run(decibelResponder: cbObject)

    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }
}
