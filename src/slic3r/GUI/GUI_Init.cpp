#include "GUI_Init.hpp"

#include "libslic3r/AppConfig.hpp"
#include "libslic3r/PerfTrace.hpp"

#include "slic3r/GUI/GUI.hpp"
#include "slic3r/GUI/GUI_App.hpp"
#include "slic3r/GUI/3DScene.hpp"
#include "slic3r/GUI/InstanceCheck.hpp"
#include "slic3r/GUI/format.hpp"
#include "slic3r/GUI/MainFrame.hpp"
#include "slic3r/GUI/Plater.hpp"

// To show a message box if GUI initialization ends up with an exception thrown.
#include <wx/msgdlg.h>

#include "libslic3r/Log.hpp"
#include <boost/nowide/iostream.hpp>
#include <boost/nowide/convert.hpp>

#ifndef _WIN32
    #include <signal.h>
#endif
#ifdef __linux__
    #include <cstdlib>
    #include <cstring>
    #include <filesystem>
    #include <fstream>
    #include <string>
#endif

namespace Slic3r {
namespace GUI {

#ifdef __linux__
// NALA-TURBO (BS-02): Runtime Intel iGPU/Arc (Mesa) & Wayland WebKitGTK DMA-BUF Guard.
// Prevents white-screens, EGL compositor deadlocks, and UI stutter on Ubuntu 22.04/24.04/26.04
// without overriding explicit user environment variables (setenv overwrite = 0).
static void nala_apply_linux_intel_gpu_guards()
{
    bool intel_gpu_detected = false;
    std::error_code ec;
    const std::filesystem::path drm_root("/sys/class/drm");
    if (std::filesystem::exists(drm_root, ec)) {
        for (const auto &entry : std::filesystem::directory_iterator(drm_root, ec)) {
            const std::filesystem::path vendor_file = entry.path() / "device" / "vendor";
            std::ifstream ifs(vendor_file);
            std::string vendor_id;
            if (ifs && std::getline(ifs, vendor_id)) {
                if (vendor_id.find("0x8086") != std::string::npos) {
                    intel_gpu_detected = true;
                    break;
                }
            }
        }
    }

    const char *keep_dmabuf   = std::getenv("NALA_KEEP_WEBKIT_DMABUF");
    const char *force_safe    = std::getenv("NALA_FORCE_SAFE_WEBKIT");
    const char *wayland_disp  = std::getenv("WAYLAND_DISPLAY");
    const char *force_wayland = std::getenv("NALA_FORCE_WAYLAND");
    const bool keep_dmabuf_on = (keep_dmabuf && std::strcmp(keep_dmabuf, "1") == 0);
    const bool force_safe_on  = (force_safe && std::strcmp(force_safe, "1") == 0);

    if (!keep_dmabuf_on && (intel_gpu_detected || force_safe_on || wayland_disp != nullptr)) {
        ::setenv("WEBKIT_DISABLE_DMABUF_RENDERER", "1", 0);
        if (intel_gpu_detected || force_safe_on)
            ::setenv("WEBKIT_DISABLE_COMPOSITING_MODE", "1", 0);
        ::setenv("MESA_SHADER_CACHE_MAX_SIZE", "512M", 0);
        if (wayland_disp != nullptr && !(force_wayland && std::strcmp(force_wayland, "1") == 0))
            ::setenv("GDK_BACKEND", "x11", 0);
    }
    perf_mark("startup.nala_intel_guard_applied");
}
#endif

int GUI_Run(GUI_InitParams &params)
{
#ifdef __linux__
    nala_apply_linux_intel_gpu_guards();
#endif
#if __APPLE__
    // On OSX, we use boost::process::spawn() to launch new instances of PrusaSlicer from another PrusaSlicer.
    // boost::process::spawn() sets SIGCHLD to SIGIGN for the child process, thus if a child PrusaSlicer spawns another
    // subprocess and the subrocess dies, the child PrusaSlicer will not receive information on end of subprocess
    // (posix waitpid() call will always fail).
    // https://jmmv.dev/2008/10/boostprocess-and-sigchld.html
    // The child instance of PrusaSlicer has to reset SIGCHLD to its default, so that posix waitpid() and similar continue to work.
    // See GH issue #5507
    signal(SIGCHLD, SIG_DFL);
#endif // __APPLE__
#ifndef _WIN32
    signal(SIGPIPE, SIG_IGN);
#endif

    //BBS: remove the try-catch and let exception goto above
    try {
        GUI::GUI_App* gui = nullptr;
        {
            PERF_TRACE("startup.create_gui_app");
            //GUI::GUI_App* gui = new GUI::GUI_App(params.start_as_gcodeviewer ? GUI::GUI_App::EAppMode::GCodeViewer : GUI::GUI_App::EAppMode::Editor);
            gui = new GUI::GUI_App();
        }
        //if (gui->get_app_mode() != GUI::GUI_App::EAppMode::GCodeViewer) {
            // G-code viewer is currently not performing instance check, a new G-code viewer is started every time.
            bool gui_single_instance_setting = gui->app_config->get("app", "single_instance") == "true";
            if (Slic3r::instance_check(params.argc, params.argv, gui_single_instance_setting)) {
                //TODO: do we have delete gui and other stuff?
                return -1;
            }
        //}

//      gui->autosave = m_config.opt_string("autosave");
        GUI::GUI_App::SetInstance(gui);
        gui->init_params = &params;

        // wxEntry blocks until the app exits; its cost covers OnInit + main loop lifetime.
        // Stage breakdown inside OnInit is logged separately under startup.* names.
        perf_mark("startup.wxEntry enter");
        if (params.argc > 1) {
            // STUDIO-273 wxWidgets report error when opening some files with specific names
            // wxWidgets does not handle parameters, so intercept parameters here, only keep the app name
            int                 argc = 1;
            std::vector<char *> argv;
            argv.push_back(params.argv[0]);
            return wxEntry(argc, argv.data());
        } else {
            return wxEntry(params.argc, params.argv);
        }
    } catch (const Slic3r::Exception &ex) {
        if (is_log_trivial_valid()) {
            BOOST_LOG_TRIVIAL(error) << ex.what() << std::endl; // boost log not initialized yet 
        }
        wxMessageBox(boost::nowide::widen(ex.what()), _L("Bambu Studio GUI initialization failed"), wxICON_STOP);

    } catch (const std::exception &ex) {
        if (is_log_trivial_valid()) {
            BOOST_LOG_TRIVIAL(error) << ex.what() << std::endl; // boost log not initialized yet 
        }
        wxMessageBox(format_wxstr(_L("Fatal error, exception caught: %1%"), ex.what()), _L("Bambu Studio GUI initialization failed"), wxICON_STOP);
    }
    // error
    return 1;
}
}
}
