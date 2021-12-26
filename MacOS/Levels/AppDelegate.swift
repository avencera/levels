//
//  AppDelegate.swift
//  Levels
//
//  Created by Praveen Perera on 12/22/22.
//  Copyright © 2021 Praveen Perera. All rights reserved.
//

import Cocoa
import SwiftUI

class DecibelResponderImpl: DecibelResponder {
    var statusBar: StatusBarController?
    
    func setStatusBar(sb: StatusBarController?) {
        statusBar = sb
    }

    func decibel(decibel: Int32) {
        statusBar?.changeText(text: String(decibel))
    }
}

@main
class AppDelegate: NSObject, NSApplicationDelegate {
    var popover = NSPopover.init()
    var statusBar: StatusBarController?
    var levels: Levels?


    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Create the SwiftUI view that provides the contents
        let contentView = ContentView()
        
        levels = Levels()

        // Set the SwiftUI's ContentView to the Popover's ContentViewController
        popover.contentViewController = MainViewController()
        popover.contentSize = NSSize(width: 360, height: 360)
        popover.contentViewController?.view = NSHostingView(rootView: contentView)
        
        
        let cbObject = DecibelResponderImpl()
        
        
        // Create the Status Bar Item with the Popover
        statusBar = StatusBarController.init(popover)
        statusBar?.changeText(text: "h2")
        
        cbObject.setStatusBar(sb: statusBar)
        levels?.run(decibelResponder: cbObject)

    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }
}
