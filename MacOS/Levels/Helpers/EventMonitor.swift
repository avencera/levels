//
//  EventMonitor.swift
//  Levels
//
//  Created by Praveen Perera on 12/22/22.
//  Copyright © 2021 Praveen Perera. All rights reserved.
//

import Cocoa

class EventMonitor {
    private var monitor: Any?
    private let mask: NSEvent.EventTypeMask
    private let handler: (NSEvent?) -> Void
    
    public init(mask: NSEvent.EventTypeMask, handler: @escaping (NSEvent?) -> Void) {
      self.mask = mask
      self.handler = handler
    }

    deinit {
      stop()
    }

    public func start() {
        monitor = NSEvent.addGlobalMonitorForEvents(matching: mask, handler: handler) as! NSObject
    }

    public func stop() {
      if monitor != nil {
        NSEvent.removeMonitor(monitor!)
        monitor = nil
      }
    }
}
