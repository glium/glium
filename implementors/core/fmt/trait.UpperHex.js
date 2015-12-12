(function() {var implementors = {};
implementors['libc'] = [];implementors['wayland_client'] = [];implementors['wayland_kbd'] = [];implementors['wayland_window'] = [];implementors['glutin'] = [];implementors['glium'] = [];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
