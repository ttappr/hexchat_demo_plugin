
#![allow(unused_variables)]

//! This is an example for how a plugin can be written using the hexchat_plugin
//! library. The hexchat_api library is still in early development.

/* EXAMPLE PLUGIN USING RUST HEXCHAT API */

extern crate hexchat_api;

use hexchat_api::*;
use hexchat_api::FieldValue::*;

use std::any::Any;
use std::thread;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc; 
use thread_id;
 
// Set this plugin's DLL entry functions. These are the functions implemented
// in this file, below.
dll_entry_points!( plugin_get_info, 
                   plugin_init, 
                   plugin_deinit );

// Plugins have to implement an info that function that returns a pinned
// `PluginInfo` object. This is used to register the plugin and display
// information on it in the "Plugins and Scripts" window, among other things.
fn plugin_get_info() -> Pin<Box<PluginInfo>>
{
    // The `PluginInfo` constructor returns a pinned/boxed instance that can be
    // returned as-is from this function.
    PluginInfo::new("RustPlugin", 
                    "0.1", 
                    "My Rust Hexchat Plugin")
}

// This is this library's version of the `hexchat_init()` exported function.
fn plugin_init(hexchat: &Hexchat) -> i32
{
    // A variable that can be moved into closures.
    let mut i = 1;

    // A closure command callback that can be registered more than once.
    let fu = move |hc       : &Hexchat, 
                   word     : &[String], 
                   word_eol : &[String], 
                   ud       : &mut Option<Box<dyn Any>>| 
            {
                hc.print("\x0313[example[5|6]]\tCopied closure.");
                hc.print(&format!("\x0311#word\t{:?}", word));
                hc.print(&format!("\x0309#word_eol\t{:?}", 
                                  word_eol));
                i += 1;
                hc.print(&format!("i = {}", i));
                Eat::All
             };

    // A callback that implements and uses user_data.
    hexchat.hook_command("Example", 
                         Priority::Norm,
                         example, 
                         "Command implemented as a static function. \
                          Prints 'word' and 'word_eol'.",
                         Some(Box::new("Hello world!!")));
    
    hexchat.hook_command("Example2",
                         Priority::Norm,
                         |hc, word, word_eol, ud| {
                            hc.print("\x0313[example2]\tYay! It works!");
                            hc.print(&format!("\x0311#word\t{:?}", word));
                            hc.print(&format!("\x0309#word_eol\t{:?}", 
                                              word_eol));
                            Eat::All
                         },
                         "An example command implemented using a closure.",
                         None);
    
    hexchat.hook_command("rustpanic",
                         Priority::Norm,
                         |hc, word, word_eol, ud| {
                            panic!("Ruh-roh!");
                         },
                         "An example command that throws a panic. \
                          The panic is 'caught' and displayed in the active \
                          window.",
                         None);

    hexchat.hook_command("Example4",
                         Priority::Norm,
                         move |hc, word, word_eol, ud| {
                            hc.print(&format!("\x0313[example4]\t {}", i));
                            i += 1;
                            Eat::All
                         },
                         "An example command that updates a variable moved \
                          into its enclosed scope.",
                         None);
                         
    hexchat.hook_command("Example5",
                         Priority::Norm,
                         fu,
                         "An example command that uses the same underlying \
                          closure as example6.",
                         None);

    hexchat.hook_command("Example6",
                         Priority::Norm,
                         fu,
                         "An example command that uses the same underlying \
                          closure as example5",
                         None);

    // Set up a shared variable to hold a hook for a timer callback.
    let shared_hook   = Rc::new(RefCell::new(None));
    let shared_hook_1 = shared_hook.clone(); // /STARTTIMER's copy.
    let shared_hook_2 = shared_hook.clone(); // /STOPTIMER's copy.

    hexchat.hook_command(
        "starttimer",
        Priority::Norm,

        move |hc, word, word_eol, ud| {
            let timer_hook = &mut *shared_hook_1.borrow_mut();

            if timer_hook.is_some() {
                hc.print("Timer is already running.");
                return Eat::All;
            }
            hc.print("\x0311[timer]\t\
                      Setting up timer callback. Issue /STOPTIMER to stop it \
                      early, or it will stop automatically after 10  times.");

            // Timer callback needs a copy of the hook var from this scope.
            let shared_hook_cb = shared_hook_1.clone();

            // Register the timer callback and retain its hook.
            let hook = hc.hook_timer(
                2000, // 2 second pause between invocations.

                move |hc, ud| {
                    if let Some(n) = ud {
                        // Shows how to mutably access the user_data.
                        let n = n.downcast_mut::<i32>().unwrap();
                        hc.print(&format!("timer user data = {}.", n));
                        *n += 1;
                        if *n > 10 {
                            let my_hook = &mut *shared_hook_cb.borrow_mut();
                            *my_hook = None;
                            return 0; // Causes timer to stop.
                        }
                    }
                    hc.print("\x0311[timer]\tTimer callback invoked.");
                    1 // Keep going.
                },

                // Mutable user data to keep track of iteration count.
                Some(Box::new(1)));

            *timer_hook = Some(hook);
            Eat::All
        },

        "Starts a timer callback that gets invoked every two seconds.",
        None);

    // Command to stop the timer callback registered above.
    hexchat.hook_command(
        "stoptimer",
        Priority::Norm,

        move |hc, word, word_eol, ud| {
            hc.print("\x0313[stoptimer]\t\
                      Stopping timer callback, and printing its user data.");

            let mut timer_user_data = None;
            let     timer_hook      = &mut *shared_hook_2.borrow_mut();

            // Unhook the timer callback and take ownership of its user_data.
            if let Some(ref timer_hook) = timer_hook {
                timer_user_data = timer_hook.unhook();
            }
            *timer_hook = None;

            if let Some(n) = timer_user_data {
                if let Some(n) = n.downcast_ref::<i32>() {
                    hc.print(&format!("timer callback's user_data: {}", n));
                }
            } else {
                hc.print("timer callback's user_data was `None`.");
                hc.print("The timer either already expired, or /STOPTIMER has \
                          already been called.");
            }
            Eat::All
        },

        "Stops the timer callback",
        None);

    // Demo how to register an object's method as a callback.                         
    let obj = MyObj::new(25);
    hexchat.hook_command("example7",
                         Priority::Norm,
                         // Simply wrap the callback method in a closure.
                         move |hc, word, word_eol, ud| {
                            obj.method_callback(hc, word, word_eol, ud)
                         },
                         "Demonstrates how to use an object's method as a \
                          callback",
                         None);

    // Shows how separate threads can safely call Hexchat's API.
    hexchat.hook_command(
        "runthread",
        Priority::Norm,

        |hc, word, word_eol, ud| {
            fn safe_print(msg: &str) {
                let rc_msg = Arc::new(msg.to_string());
                main_thread(move |hc| hc.print(&rc_msg));
            }
            // Spawn a new thread.
            thread::spawn(|| {
                let tid = thread_id::get();
                safe_print(&format!("{}[spawned-thread]\t\
                                    Hello, from spawned thread {}.",
                                    "\x0313", tid));
                // Send a task to the main thread to have executed and get its
                // AsyncResult object.
                let ar = main_thread(
                    move |hc| {
                        let main_tid = thread_id::get();
                        hc.print(&format!("{}[main-thread]\t\
                                           Hello, from main thread {}.",
                                          "\x0313", main_tid));
                        // Return data to the calling thread.
                        format!("{}THREAD {} RETURNED THIS DATA TO THREAD {}.",
                                "\x0311", main_tid, tid)
                    });
                // Get the return data from the main thread callback (blocks).
                let r = ar.get();
                safe_print(&format!("{}[spawned-thread]\t\
                                     The previous command ran on the main \
                                     thread and returned this string: {}",
                                    "\x0313", r));
            });
            Eat::All
        },

        "Runs a new thread that sets up a closure to run on the main thread.",
        None);

    // A command that lists the user info for all the users in a channel.
    // Demonstrates use of the `ListIterator` to access the various lists
    // of Hexchat.
    hexchat.hook_command(
        "userinfo",
        Priority::Norm,

        |hc, word, word_eol, ud| {
            hc.print("Channel User List");
            hc.print("-----------------");

            if let Some(list) = ListIterator::new("users") {
                let mut count = 0;
                let mut val;
                let     fields = list.get_field_names();

                for item in &list {
                    count += 1;
                    let user_name = {
                        if let StringVal(n) = item.get_field("nick").unwrap()
                        { n } else { "????".to_string() } };

                    hc.print(&format!("[{}]", user_name));

                    for field in fields {
                        match item.get_field(&field).unwrap() {
                            StringVal(s) => { val = s;                    },
                            IntVal(i)    => { val = i.to_string();        },
                            any          => { val = format!("{:?}", any); },
                        }
                        hc.print(&format!("    {:10}: {}", field, val));
                    }
                }
                if count != 0 {
                    hc.print(&format!("Listed {} members in this channel.",
                                      count));
                } else {
                    hc.print("Looks like there were no users to list. \
                              This can happen with private message channels.");
                }
            } else {
                hc.print("Unable to retrieve user list for channel.");
            }
            Eat::All
        },

        "Prints info for each user in a channel.",
        None);

    // This shows a push model approach to traversing the user list.
    hexchat.hook_command(
        "userinfo2",
        Priority::Norm,

        |hc, word, word_eol, ud| {
            if let Some(list) = ListIterator::new("users") {
                list.traverse(
                    // `traverse()` takes a visitor callback to receive data.
                    |field_name, value, is_new_rec| {
                        if is_new_rec {
                            hc.print("-----------------------------");
                        }
                        hc.print(&format!("{:10}: {:?}", field_name, value));

                        true // Keep going.
                    })
            }

            Eat::All
        },

        "Lists user info using a different approach (`traverse()`).",
        None);

    // The event type the following commands will use to emit and handle
    // a Hexchat event issued with `emit_print()`.
    let event_type = "Generic Message";

    // Set up the event receiver.
    hexchat.hook_print(
        event_type,
        Priority::Highest,

        |hc, word, ud| {
            hc.print(&format!("\x0313[receive-event]\t\
                               Received word data: {:?}", word));
            Eat::All
        },

        None);

    // Register a command to send the event the receiver is waiting on.
    hexchat.hook_command(
        "emitevent",
        Priority::Norm,

        move |hc, word, word_eol, ud| {
            let args: Vec<_> = word.iter().map(String::as_str).collect();
            let slice = &args[1..];

            hc.print(&format!("\x0311[send-event]\t\
                               Invoking: `hc.emit_print(\"{}\", &{:?})`",
                              event_type, slice));
            // Send it!
            if let Err(err) = hc.emit_print(event_type, slice) {
                hc.print(&format!("{}", err));
            } else {}

            Eat::All
        },

        "Sends event using `hexchat.emit_print()`.",
        None);

    1
}

// Called when plugin is unloaded.
fn plugin_deinit(hexchat: &Hexchat) -> i32 
{
    1
}

// A command callback implemented as a typical function.
fn example (hc          : &Hexchat,
            word        : &[String],
            word_eol    : &[String],
            user_data   : &mut Option<Box<dyn Any>> 
           ) -> Eat 
{
    hc.print("\x0313[example]\tExecuting example command.");
    hc.print(&format!("\x0311#word\t{:?}", word));
    hc.print(&format!("\x0309#word_eol\t{:?}", word_eol));
    
    // How to access data within user_data.
    if let Some(ud) = user_data {
        if let Some(msg) = ud.downcast_ref::<&str>() {
            hc.print(&format!("User data received: {:?}", msg));
        } else {
            hc.print("Received user_data, but it's not a &str");
        }
    }
    Eat::All
}

// An object that implements a command callback.
struct MyObj {
    data: i32,
}
impl MyObj {
    fn new(data: i32) -> Self {
        MyObj { data }
    }
    // Wrap this in a closure when registering it as a command callback.
    fn method_callback(&self,
                       hc        : &Hexchat, 
                       word      : &[String],
                       word_eol  : &[String],
                       user_data : &mut Option<Box<dyn Any>>
                      ) -> Eat
    {
        hc.print(&format!("\x0311[MyObj.method]\t\
                           Called an object method! self.data = {}.", 
                           self.data));
        Eat::All
    }
}

