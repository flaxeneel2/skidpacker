package dev.skidpacker.testjar;

import javax.swing.*;

/*
Class which contains functions to display a popup.
This also acts as a test class to see if the loader has delivered the classes successfully.
 */
public class JPane {

        // Awesome message... :)
       private static String message = "================================================================================\n" +
            "Skidpacker has ran successfully! :)\n" +
            "================================================================================\n";

       // Shows the pane.
        public static void showDisplay() {
            JOptionPane.showMessageDialog(null, message);
        }
}
