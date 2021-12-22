//
//  MainViewController.swift
//  Levels
//
//  Created by Praveen Perera on 12/22/22.
//  Copyright © 2021 Praveen Perera. All rights reserved.
//

import AppKit

class MainViewController: NSViewController {
    override func viewDidAppear()
    {
        super.viewDidAppear()

        // You can use a notification and observe it in a view model where you want to fetch the data for your SwiftUI view every time the popover appears.
        // NotificationCenter.default.post(name: Notification.Name("ViewDidAppear"), object: nil)
    }
}
