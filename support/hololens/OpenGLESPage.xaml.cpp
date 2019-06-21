#include "pch.h"
#include "OpenGLESPage.xaml.h"
#include "Servo.h"

using namespace hlservo;
using namespace Platform;
using namespace Concurrency;
using namespace Windows::Foundation;

static char sWakeupEvent[] = "SIGNAL_WAKEUP";

OpenGLESPage::OpenGLESPage()
    : OpenGLESPage(nullptr)
{
}

OpenGLESPage::OpenGLESPage(OpenGLES* openGLES)
    : mOpenGLES(openGLES)
    , mRenderSurface(EGL_NO_SURFACE)
{
    InitializeComponent();
    Windows::UI::Core::CoreWindow ^ window = Windows::UI::Xaml::Window::Current->CoreWindow;
    window->VisibilityChanged += ref new Windows::Foundation::TypedEventHandler<Windows::UI::Core::CoreWindow ^, Windows::UI::Core::VisibilityChangedEventArgs ^>(this, &OpenGLESPage::OnVisibilityChanged);
    this->Loaded += ref new Windows::UI::Xaml::RoutedEventHandler(this, &OpenGLESPage::OnPageLoaded);
}

OpenGLESPage::~OpenGLESPage()
{
    StopRenderLoop();
    DestroyRenderSurface();
}

void OpenGLESPage::OnPageLoaded(Platform::Object ^ sender, Windows::UI::Xaml::RoutedEventArgs ^ e)
{
    CreateRenderSurface();
    StartRenderLoop();
}

void OpenGLESPage::OnVisibilityChanged(Windows::UI::Core::CoreWindow ^ sender, Windows::UI::Core::VisibilityChangedEventArgs ^ args)
{
    if (args->Visible && mRenderSurface != EGL_NO_SURFACE) {
        StartRenderLoop();
    } else {
        StopRenderLoop();
    }
}

void OpenGLESPage::CreateRenderSurface()
{
    if (mOpenGLES && mRenderSurface == EGL_NO_SURFACE) {
        mRenderSurface = mOpenGLES->CreateSurface(swapChainPanel);
    }
}

void OpenGLESPage::DestroyRenderSurface()
{
    if (mOpenGLES) {
        mOpenGLES->DestroySurface(mRenderSurface);
    }
    mRenderSurface = EGL_NO_SURFACE;
}

void OpenGLESPage::RecoverFromLostDevice()
{
    StopRenderLoop();
    {
        critical_section::scoped_lock lock(mRenderSurfaceCriticalSection);

        DestroyRenderSurface();
        mOpenGLES->Reset();
        CreateRenderSurface();
    }
    StartRenderLoop();
}

void OpenGLESPage::StartRenderLoop()
{
    if (mRenderLoopWorker != nullptr && mRenderLoopWorker->Status == Windows::Foundation::AsyncStatus::Started) {
        return;
    }

    auto loop = [this](Windows::Foundation::IAsyncAction ^ action) {
      critical_section::scoped_lock lock(mRenderSurfaceCriticalSection);

      HANDLE hEvent = ::CreateEventA(nullptr, FALSE, FALSE, sWakeupEvent);

      // Called by Servo
      Servo::sMakeCurrent = [this]() {
        /* EGLint panelWidth = 0; */
        /* EGLint panelHeight = 0; */
        /* mOpenGLES->GetSurfaceDimensions(mRenderSurface, &panelWidth, &panelHeight); */
        /* glViewport(0, 0, panelWidth, panelHeight); */
        /* mServo->SetSize(panelWidth, panelHeight); */
        mOpenGLES->MakeCurrent(mRenderSurface);
      };

      // Called by Servo
      Servo::sFlush = [this]() {
        if (mOpenGLES->SwapBuffers(mRenderSurface) != GL_TRUE) {
          // The call to eglSwapBuffers might not be successful (i.e. due to Device Lost)
          // If the call fails, then we must reinitialize EGL and the GL resources.
          swapChainPanel->Dispatcher->RunAsync(Windows::UI::Core::CoreDispatcherPriority::High, ref new Windows::UI::Core::DispatchedHandler([=]() {
                RecoverFromLostDevice();
          }, CallbackContext::Any));
        }
      };

      mOpenGLES->MakeCurrent(mRenderSurface);

      EGLint panelWidth = 0;
      EGLint panelHeight = 0;
      mOpenGLES->GetSurfaceDimensions(mRenderSurface, &panelWidth, &panelHeight);
      glViewport(0, 0, panelWidth, panelHeight);
      mServo = new Servo(panelWidth, panelHeight);

      while (action->Status == Windows::Foundation::AsyncStatus::Started) {
        // Block until Servo::sWakeUp is called.
        // Or run full speed if animating (see on_animating_changed),
        // it will endup blocking on SwapBuffers to limit rendering to 60FPS
        if (!Servo::sAnimating) {
           ::WaitForSingleObject(hEvent, INFINITE);
        }
        mServo->PerformUpdates();
      }
    };

    auto workItemHandler = ref new Windows::System::Threading::WorkItemHandler(loop);

    // Run Servo task in a high priority background thread.
    mRenderLoopWorker = Windows::System::Threading::ThreadPool::RunAsync(
      workItemHandler,
      Windows::System::Threading::WorkItemPriority::High,
      Windows::System::Threading::WorkItemOptions::TimeSliced);

    Servo::sWakeUp = []() {
      HANDLE hEvent = ::OpenEventA(EVENT_ALL_ACCESS, FALSE, sWakeupEvent);
      ::SetEvent(hEvent);
    };
}

void OpenGLESPage::StopRenderLoop()
{
    if (mRenderLoopWorker) {
        mRenderLoopWorker->Cancel();
        mRenderLoopWorker = nullptr;
    }
}
