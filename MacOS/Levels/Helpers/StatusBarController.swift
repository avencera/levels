//
//  StatusBarController.swift
//  Levels
//
//  Created by Praveen Perera on 12/22/22.
//  Copyright © 2021 Praveen Perera. All rights reserved.
//

import AppKit

class StatusBarController {
    private var statusBar: NSStatusBar
    private var statusItem: NSStatusItem
    private var popover: NSPopover
    private var eventMonitor: EventMonitor?
    
    init(_ popover: NSPopover)
    {
        self.popover = popover
        statusBar = NSStatusBar.init()
        statusItem = statusBar.statusItem(withLength: 28.0)
        
        if let statusBarButton = statusItem.button {
            statusBarButton.bezelStyle = .texturedSquare
            statusBarButton.isBordered = false
            statusBarButton.wantsLayer = true
            statusBarButton.title = ""
            statusBarButton.action = #selector(togglePopover(sender:))
            statusBarButton.target = self
        }
        
        eventMonitor = EventMonitor(mask: [.leftMouseDown, .rightMouseDown], handler: mouseEventHandler)
    }
    
    @objc func togglePopover(sender: AnyObject) {
        if(popover.isShown) {
            hidePopover(sender)
        }
        else {
            showPopover(sender)
        }
    }
    
    public func changeText(text: String, color: Color) {
        let color: CGColor = {
            switch color {
            case Color.red:
                return NSColor.red.cgColor
            case Color.blue:
                return NSColor.blue.cgColor
            case Color.green:
                return NSColor.green.cgColor
            case Color.yellow:
                return NSColor.systemYellow.cgColor
            case Color.skyBlue:
                return NSColor.init(displayP3Red: 0.502, green: 0.855, blue: 235, alpha: 0.922).cgColor
            }
        }()
        
        
        statusItem.button?.layer?.backgroundColor = color
        statusItem.button?.title = text
    }
    
    func showPopover(_ sender: AnyObject) {
        if let statusBarButton = statusItem.button {
            popover.show(relativeTo: statusBarButton.bounds, of: statusBarButton, preferredEdge: NSRectEdge.maxY)
            eventMonitor?.start()
        }
    }
    
    func hidePopover(_ sender: AnyObject) {
        popover.performClose(sender)
        eventMonitor?.stop()
    }
    
    func mouseEventHandler(_ event: NSEvent?) {
        if(popover.isShown) {
            hidePopover(event!)
        }
    }
}
