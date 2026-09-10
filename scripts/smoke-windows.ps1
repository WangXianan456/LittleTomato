param(
    [string]$Executable = 'D:\Coding\Build\Cargo\debug\little-tomato.exe',
    [ValidateRange(0, 50)][int]$ColdStartRounds = 0,
    [switch]$SettingsChecks
)
$ErrorActionPreference = 'Stop'
# Sends messages to the test application's HWND; does not lock or suspend Windows.
if (Get-Process little-tomato -ErrorAction SilentlyContinue) { throw 'Close Little Tomato before this isolated smoke test.' }
. "$PSScriptRoot/dev-env.ps1"
$normalRuntime = $env:LITTLE_TOMATO_RUNTIME_DIR
$env:LITTLE_TOMATO_RUNTIME_DIR = Join-Path 'D:\Coding\Runtime' ('LittleTomato-smoke-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $env:LITTLE_TOMATO_RUNTIME_DIR | Out-Null
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TomatoSmoke {
 public delegate bool EnumProc(IntPtr hwnd, IntPtr param);
 [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc callback, IntPtr param);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr hwnd, System.Text.StringBuilder text, int length);
 public static IntPtr Find(uint pid, string title) {
  IntPtr found=IntPtr.Zero;
  EnumWindows((h,p)=> { uint owner;GetWindowThreadProcessId(h,out owner); if(owner==pid) { var text=new System.Text.StringBuilder(256);GetWindowText(h,text,256);if(text.ToString()==title) {found=h;return false;} } return true; },IntPtr.Zero);
  return found;
 }
 [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X,Y; public POINT(int x,int y) { X=x;Y=y; } }
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT point);
 [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr hwnd,uint flags);
 public static System.Diagnostics.Process[] LaunchPair(string executable) {
  var start = new System.Threading.ManualResetEvent(false);
  var processes = new System.Diagnostics.Process[2];
  var failures = new Exception[2];
  var threads = new System.Threading.Thread[2];
  for (int i = 0; i < 2; i++) {
   int index = i;
   threads[i] = new System.Threading.Thread(() => {
    start.WaitOne();
    try { processes[index] = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(executable) {
     UseShellExecute = false, CreateNoWindow = true, WindowStyle = System.Diagnostics.ProcessWindowStyle.Hidden
    }); } catch (Exception error) { failures[index] = error; }
   });
   threads[i].Start();
  }
  start.Set();
  foreach (var thread in threads) thread.Join();
  start.Dispose();
  if (failures[0] != null || failures[1] != null) {
   foreach (var process in processes) if (process != null && !process.HasExited) process.Kill();
   throw new InvalidOperationException("Cold start launch failed", failures[0] ?? failures[1]);
  }
  return processes;
 }
 [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left,Top,Right,Bottom; }
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out RECT rect);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
 [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeout(IntPtr hwnd,uint message,UIntPtr wparam,IntPtr lparam,uint flags,uint timeout,out UIntPtr result);
}
'@
function Assert-True($value, $description) {
    if (-not $value) { throw "FAILED: $description" }
    Write-Output "PASS: $description"
}
function Read-Snapshot {
    $json = @'
import { DatabaseSync } from 'node:sqlite';
import path from 'node:path';
const db=new DatabaseSync(path.join(process.env.LITTLE_TOMATO_RUNTIME_DIR,'data/little-tomato.sqlite3'),{readOnly:true});
db.exec('PRAGMA busy_timeout=5000');
console.log(db.prepare('SELECT snapshot FROM timer_state WHERE id=1').get().snapshot);
db.close();
'@ | node --input-type=module
    if ($LASTEXITCODE -ne 0) { throw 'Could not read test snapshot' }
    $json | ConvertFrom-Json
}
function Send-Event([uint32]$message, [uint32]$value) {
    $result=[UIntPtr]::Zero
    $sent=[TomatoSmoke]::SendMessageTimeout($script:window,$message,[UIntPtr]$value,[IntPtr]::Zero,2,5000,[ref]$result)
    if ($sent -eq [IntPtr]::Zero) { throw 'Native message timed out' }
    Start-Sleep -Milliseconds 250
}
function Click-Primary {
    $rect=New-Object TomatoSmoke+RECT
    [TomatoSmoke]::GetWindowRect($script:window,[ref]$rect) | Out-Null
    $scale=($rect.Right-$rect.Left)/300.0
    [TomatoSmoke]::SetCursorPos(($rect.Left+[int](188*$scale)),($rect.Top+[int](347*$scale))) | Out-Null
    Start-Sleep -Milliseconds 150
    [TomatoSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
    [TomatoSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 1200
}
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
function Main-Control([string]$name) {
    $root=[System.Windows.Automation.AutomationElement]::FromHandle($script:window)
    $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)))
}
function Toggle-Details {
    $root=[System.Windows.Automation.AutomationElement]::FromHandle($script:window)
    $pet=$root.FindAll([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty,[System.Windows.Automation.ControlType]::Button))) | Where-Object {$_.Current.Name -like '和*说话；按住拖动可移动'} | Select-Object -First 1
    $r=$pet.Current.BoundingRectangle
    [TomatoSmoke]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
    Start-Sleep -Milliseconds 250
    for($click=0;$click -lt 2;$click++) {
        [TomatoSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        Start-Sleep -Milliseconds 40
        [TomatoSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
        Start-Sleep -Milliseconds 80
    }
    Start-Sleep -Milliseconds 500
}
$first=$null
$duplicate=$null
$pair=@()
$evidenceRoot=$env:LITTLE_TOMATO_RUNTIME_DIR
$previousDpi=[TomatoSmoke]::SetThreadDpiAwarenessContext([IntPtr](-4))
Start-Transcript -Path (Join-Path $evidenceRoot 'smoke.log') | Out-Null
try {
    for ($round=1; $round -le $ColdStartRounds; $round++) {
        $env:LITTLE_TOMATO_RUNTIME_DIR=Join-Path $evidenceRoot "cold-$round"
        $pair=[TomatoSmoke]::LaunchPair($Executable)
        Start-Sleep -Seconds 3
        $survivors=@($pair | Where-Object { -not $_.HasExited })
        Assert-True ($survivors.Count -eq 1) "Cold start round $round has one survivor"
        $survivors[0].Refresh()
        Assert-True ($survivors[0].MainWindowHandle -ne [IntPtr]::Zero -and $survivors[0].Responding) "Cold start round $round has a responsive main window"
        Assert-True ((Read-Snapshot).status -eq 'ready') "Cold start round $round initializes readable ready state"
        foreach ($owned in $pair) { if (-not $owned.HasExited) { Stop-Process -Id $owned.Id; $owned.WaitForExit() } }
        $pair=@()
    }
    $env:LITTLE_TOMATO_RUNTIME_DIR=$evidenceRoot
    $first=Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
    Start-Sleep -Seconds 3
    $first.Refresh()
    $script:window=$first.MainWindowHandle
    Assert-True ($script:window -ne [IntPtr]::Zero) 'Main window created'
    $bounds=New-Object TomatoSmoke+RECT
    [TomatoSmoke]::GetWindowRect($script:window,[ref]$bounds) | Out-Null
    $dpi=[TomatoSmoke]::GetDpiForWindow($script:window)
    Write-Output "DPI: $dpi; bounds: $($bounds.Left),$($bounds.Top),$($bounds.Right),$($bounds.Bottom)"
    Assert-True (($bounds.Right-$bounds.Left) -eq [int](300*$dpi/96) -and ($bounds.Bottom-$bounds.Top) -eq [int](420*$dpi/96)) 'Window dimensions match actual DPI'
    Add-Type -AssemblyName System.Windows.Forms
    $work=[System.Windows.Forms.Screen]::FromHandle($script:window).WorkingArea
    Assert-True ($bounds.Left -ge $work.Left -and $bounds.Top -ge $work.Top -and $bounds.Right -le $work.Right -and $bounds.Bottom -le $work.Bottom) 'Window stays inside monitor work area'
    Assert-True ($null -eq (Main-Control '计时器') -and $null -eq (Main-Control '打开设置')) 'Startup shows only pet without information controls'
    [TomatoSmoke]::SetCursorPos(($bounds.Left+[int](188*$dpi/96)),($bounds.Top+[int](347*$dpi/96))) | Out-Null
    Start-Sleep -Milliseconds 300
    $compactHit=[TomatoSmoke]::GetAncestor([TomatoSmoke]::WindowFromPoint([TomatoSmoke+POINT]::new(($bounds.Left+[int](188*$dpi/96)),($bounds.Top+[int](347*$dpi/96)))),2)
    Assert-True ($compactHit -ne $script:window) 'Hidden information region passes mouse through'
    Toggle-Details
    Assert-True ($null -ne (Main-Control '计时器')) 'Double click reveals information controls'
    foreach ($target in @(@(4,4,$false),@(150,225,$true),@(188,347,$true))) {
        $x=$bounds.Left+[int]($target[0]*$dpi/96)
        $y=$bounds.Top+[int]($target[1]*$dpi/96)
        [TomatoSmoke]::SetCursorPos($x,$y) | Out-Null
        Start-Sleep -Milliseconds 200
        $hit=[TomatoSmoke]::GetAncestor([TomatoSmoke]::WindowFromPoint([TomatoSmoke+POINT]::new($x,$y)),2)
        Assert-True (($hit -eq $script:window) -eq $target[2]) "Hit target $($target[0]),$($target[1]) matches visible region at $dpi DPI"
    }
    Click-Primary
    Assert-True ((Read-Snapshot).status -eq 'running') 'Real UI click starts focus'
    Toggle-Details
    Assert-True ($null -eq (Main-Control '计时器') -and (Read-Snapshot).status -eq 'running') 'Double click hides controls without pausing timer'
    Toggle-Details
    $duplicate=Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
    Assert-True ($duplicate.WaitForExit(5000)) 'Duplicate process exits'
    Assert-True ((Read-Snapshot).status -eq 'running') 'Duplicate does not reset active timer'
    Send-Event 0x2b1 7
    $locked=Read-Snapshot
    Assert-True ($locked.status -eq 'paused' -and $locked.recoveryReason -eq 'locked') 'Session lock persists pause and reason'
    Click-Primary
    Assert-True ((Read-Snapshot).status -eq 'paused') 'Cannot resume while session is locked'
    Send-Event 0x218 4
    Send-Event 0x218 18
    Click-Primary
    Assert-True ((Read-Snapshot).status -eq 'paused') 'Power resume does not bypass session lock'
    Send-Event 0x2b1 8
    Assert-True ((Read-Snapshot).status -eq 'paused') 'Unlock requires explicit resume'
    Click-Primary
    Assert-True ((Read-Snapshot).status -eq 'running') 'User resumes after unlock'
    Send-Event 0x218 4
    $sleep=Read-Snapshot
    Assert-True ($sleep.status -eq 'paused' -and $sleep.recoveryReason -eq 'sleep') 'Suspend persists pause and reason'
    Send-Event 0x218 18
    Assert-True ((Read-Snapshot).elapsedMs -eq $sleep.elapsedMs) 'Wake adds no focus time'
    Add-Type -AssemblyName System.Drawing
    $rect=New-Object TomatoSmoke+RECT
    [TomatoSmoke]::GetWindowRect($script:window,[ref]$rect) | Out-Null
    $bitmap=New-Object System.Drawing.Bitmap ($rect.Right-$rect.Left),($rect.Bottom-$rect.Top)
    $graphics=[System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($rect.Left,$rect.Top,0,0,$bitmap.Size)
    $bitmap.Save((Join-Path $env:LITTLE_TOMATO_RUNTIME_DIR 'recovery.png'))
    $graphics.Dispose(); $bitmap.Dispose()
    Send-Event 0x10 0
    Assert-True (-not [TomatoSmoke]::IsWindowVisible($script:window)) 'Close hides to tray'
    $duplicate=Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
    Assert-True ($duplicate.WaitForExit(5000)) 'Reopen exits secondary process'
    Assert-True ([TomatoSmoke]::IsWindowVisible($script:window)) 'Reopen restores hidden instance'
    Send-Event 0 0
    Assert-True (@(Get-Process little-tomato).Count -eq 1) 'Exactly one process remains'
    if ($SettingsChecks) {
        Add-Type -AssemblyName UIAutomationClient
        Add-Type -AssemblyName UIAutomationTypes
        function Click-SettingsEntry {
            if ($null -eq (Main-Control '计时器')) { Toggle-Details }
            $r=New-Object TomatoSmoke+RECT
            [TomatoSmoke]::GetWindowRect($script:window,[ref]$r) | Out-Null
            $scale=($r.Right-$r.Left)/300.0
            $entryBounds=(Main-Control '打开设置').Current.BoundingRectangle
            [TomatoSmoke]::SetCursorPos([int]($entryBounds.X+$entryBounds.Width/2),[int]($entryBounds.Y+$entryBounds.Height/2)) | Out-Null
            Start-Sleep -Milliseconds 200
            [TomatoSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
            Start-Sleep -Milliseconds 60
            [TomatoSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
            Start-Sleep -Milliseconds 600
            $settingsWindow=[TomatoSmoke]::Find($first.Id,'小番茄 · 设置')
            if (-not [TomatoSmoke]::IsWindowVisible($settingsWindow)) {
                $petRoot=[System.Windows.Automation.AutomationElement]::FromHandle($script:window)
                $entry=$petRoot.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,'打开设置')))
                $entry.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
                Start-Sleep -Milliseconds 600
            }
        }
        function Find-Control($type, [string]$name) {
            $root=[System.Windows.Automation.AutomationElement]::FromHandle($script:panel)
            $controls=$root.FindAll([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty,$type)))
            $result=$controls | Where-Object { $_.Current.Name -like "$name*" } | Select-Object -First 1
            if (-not $result) { throw "Missing settings control: $name" }
            return $result
        }
        function Press-Button([string]$name) {
            $button=Find-Control ([System.Windows.Automation.ControlType]::Button) $name
            $deadline=[DateTime]::UtcNow.AddSeconds(3)
            while (-not $button.Current.IsEnabled -and [DateTime]::UtcNow -lt $deadline) {
                Start-Sleep -Milliseconds 100
                $button=Find-Control ([System.Windows.Automation.ControlType]::Button) $name
            }
            $pattern=$null
            if ($button.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern,[ref]$pattern)) {
                $pattern.Invoke()
            } else {
                $button.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
            }
            Start-Sleep -Milliseconds 400
        }
        function Set-FocusMinutes([string]$value) {
            $input=Find-Control ([System.Windows.Automation.ControlType]::Spinner) '专注时长'
            $deadline=[DateTime]::UtcNow.AddSeconds(5)
            while (-not $input.Current.IsEnabled -and [DateTime]::UtcNow -lt $deadline) {
                Start-Sleep -Milliseconds 100
                $input=Find-Control ([System.Windows.Automation.ControlType]::Spinner) '专注时长'
            }
            $input.GetCurrentPattern([System.Windows.Automation.RangeValuePattern]::Pattern).SetValue([double]$value)
            Start-Sleep -Milliseconds 300
        }
        function Read-Settings {
            $json=@'
import {DatabaseSync} from 'node:sqlite'; import path from 'node:path';
const db=new DatabaseSync(path.join(process.env.LITTLE_TOMATO_RUNTIME_DIR,'data/little-tomato.sqlite3'),{readOnly:true});
db.exec('PRAGMA busy_timeout=5000'); console.log(db.prepare("SELECT value FROM settings WHERE key='app_settings'").get().value); db.close();
'@ | node --input-type=module
            if ($LASTEXITCODE -ne 0) { throw 'Could not read saved settings' }
            $json | ConvertFrom-Json
        }
        Click-SettingsEntry
        $script:panel=[TomatoSmoke]::Find($first.Id,'小番茄 · 设置')
        Assert-True ($script:panel -ne [IntPtr]::Zero -and [TomatoSmoke]::IsWindowVisible($script:panel)) 'Pet settings button opens ordinary settings window'
        Set-FocusMinutes '0'
        $save=Find-Control ([System.Windows.Automation.ControlType]::Button) '保存设置'
        Assert-True (-not $save.Current.IsEnabled) 'Invalid duration cannot be saved from UI'
        Set-FocusMinutes '40'
        Press-Button '保存设置'
        Assert-True ((Read-Settings).focusMinutes -eq 40) 'UI saves custom focus duration to SQLite'
        Assert-True ((Read-Snapshot).plannedSeconds -eq 1500) 'Settings leave paused timer plan unchanged'
        Press-Button '伙伴小屋'
        foreach ($character in @(@('peach','蜜桃桃'), @('sprout','芽芽'), @('cloud','云朵'), @('cat','奶油猫'))) {
            Press-Button $character[1]
            Press-Button '保存设置'
            Assert-True ((Read-Settings).character -eq $character[0]) "Character persists: $($character[0])"
            $petRoot=[System.Windows.Automation.AutomationElement]::FromHandle($script:window)
            $petName="和$($character[1])说话；按住拖动可移动"
            $petControl=$petRoot.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$petName)))
            Assert-True ($null -ne $petControl) "Desktop receives character: $($character[0])"
            $r=New-Object TomatoSmoke+RECT
            [TomatoSmoke]::GetWindowRect($script:window,[ref]$r) | Out-Null
            $bitmap=New-Object System.Drawing.Bitmap ($r.Right-$r.Left),($r.Bottom-$r.Top)
            $graphics=[System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.CopyFromScreen($r.Left,$r.Top,0,0,$bitmap.Size)
            $bitmap.Save((Join-Path $evidenceRoot "character-$($character[0]).png"))
            $graphics.Dispose(); $bitmap.Dispose()
        }
        Assert-True ((Read-Snapshot).plannedSeconds -eq 1500) 'Character switching preserves paused timer'
        Press-Button '云朵'
        Press-Button '取消'
        Assert-True ((Read-Settings).character -eq 'cat') 'Cancel does not persist draft character'
        Click-SettingsEntry
        Press-Button '外观与陪伴'
        $motion=Find-Control ([System.Windows.Automation.ControlType]::CheckBox) '减少动画'
        $motion.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
        $slider=Find-Control ([System.Windows.Automation.ControlType]::Slider) '桌宠大小'
        $slider.GetCurrentPattern([System.Windows.Automation.RangeValuePattern]::Pattern).SetValue(240)
        $theme=Find-Control ([System.Windows.Automation.ControlType]::ComboBox) '界面主题'
        $theme.SetFocus(); Start-Sleep -Milliseconds 200
        [System.Windows.Forms.SendKeys]::SendWait('{END}{ENTER}{TAB}')
        Start-Sleep -Milliseconds 250
        Press-Button '保存设置'
        $preferences=Read-Settings
        Assert-True ($preferences.reducedMotion -and $preferences.petSize -eq 240 -and $preferences.theme -eq 'dark') 'Appearance controls save size, theme and animation preference'
        $newBounds=New-Object TomatoSmoke+RECT
        [TomatoSmoke]::GetWindowRect($script:window,[ref]$newBounds) | Out-Null
        Assert-True (($newBounds.Bottom-$newBounds.Top) -eq [int](508*$dpi/96)) 'Pet window grows with saved size'
        $r=New-Object TomatoSmoke+RECT
        [TomatoSmoke]::GetWindowRect($script:panel,[ref]$r) | Out-Null
        $bitmap=New-Object System.Drawing.Bitmap ($r.Right-$r.Left),($r.Bottom-$r.Top)
        $graphics=[System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($r.Left,$r.Top,0,0,$bitmap.Size)
        $bitmap.Save((Join-Path $evidenceRoot 'settings-dark.png'))
        $graphics.Dispose(); $bitmap.Dispose()
        [System.Windows.Forms.SendKeys]::SendWait('{ESC}')
        Start-Sleep -Milliseconds 250
        Assert-True (-not [TomatoSmoke]::IsWindowVisible($script:panel)) 'Escape hides settings panel'
        Click-SettingsEntry
        Assert-True ([TomatoSmoke]::Find($first.Id,'小番茄 · 设置') -eq $script:panel) 'Repeated opening reuses settings window'
        Stop-Process -Id $first.Id; $first.WaitForExit()
        $first=Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
        Start-Sleep -Seconds 3
        $first.Refresh(); $script:window=$first.MainWindowHandle
        Click-SettingsEntry
        $script:panel=[TomatoSmoke]::Find($first.Id,'小番茄 · 设置')
        Assert-True ((Read-Settings).focusMinutes -eq 40 -and (Read-Settings).theme -eq 'dark') 'Restart preserves settings'
        Assert-True ((Read-Settings).character -eq 'cat') 'Restart preserves character'
        $duration=Find-Control ([System.Windows.Automation.ControlType]::Spinner) '专注时长'
        Assert-True ($duration.GetCurrentPattern([System.Windows.Automation.RangeValuePattern]::Pattern).Current.Value -eq 40) 'Reopened panel renders persisted duration'
        Press-Button '恢复推荐设置'
        Press-Button '保存设置'
        Assert-True ((Read-Settings).focusMinutes -eq 25 -and (Read-Settings).petSize -eq 160 -and -not (Read-Settings).reducedMotion) 'Restore recommended settings works through UI'
        Assert-True ((Read-Settings).character -eq 'tomato') 'Restore recommended settings restores tomato'
        Press-Button '取消'
        function Find-PetControl([string]$name) {
            $petRoot=[System.Windows.Automation.AutomationElement]::FromHandle($script:window)
            return $petRoot.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)))
        }
        function Press-PetControl([string]$name) {
            $control=Find-PetControl $name
            if (-not $control) { throw "Missing pet control: $name" }
            # Transparent pets are click-through until the pointer enters a hit area.
            $bounds=$control.Current.BoundingRectangle
            [TomatoSmoke]::SetCursorPos([int]($bounds.X+$bounds.Width/2),[int]($bounds.Y+$bounds.Height/2)) | Out-Null
            Start-Sleep -Milliseconds 250
            [TomatoSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
            Start-Sleep -Milliseconds 60
            [TomatoSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
            Start-Sleep -Milliseconds 300
        }
        Press-PetControl '打开互动'
        Assert-True ($null -ne (Find-PetControl '打招呼')) 'Interaction entry opens action panel'
        Press-PetControl '打招呼'
        Assert-True ($null -eq (Find-PetControl '打招呼')) 'Choosing interaction closes action panel'
        Assert-True ((Read-Snapshot).status -eq 'paused') 'Interaction preserves paused timer'
        Press-PetControl '继续'
        Start-Sleep -Milliseconds 600
        Press-PetControl '打开互动'
        Press-PetControl '伸懒腰'
        Assert-True ($null -ne (Find-PetControl '伸伸小手，我们一起放松一下。')) 'Interaction shows character response'
        Assert-True ((Read-Snapshot).status -eq 'running') 'Interaction leaves running timer active'
        Assert-True ($null -ne (Find-PetControl '本轮进度')) 'Timer exposes accessible progress'
        $r=New-Object TomatoSmoke+RECT
        [TomatoSmoke]::GetWindowRect($script:window,[ref]$r) | Out-Null
        $bitmap=New-Object System.Drawing.Bitmap ($r.Right-$r.Left),($r.Bottom-$r.Top)
        $graphics=[System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($r.Left,$r.Top,0,0,$bitmap.Size)
        $bitmap.Save((Join-Path $evidenceRoot 'interaction-stretch.png'))
        $graphics.Dispose(); $bitmap.Dispose()
        Press-PetControl '提前结束并重置'
        Assert-True ($null -ne (Find-PetControl '结束这一轮？')) 'Reset asks before discarding active phase'
        Press-PetControl '继续这一轮'
        Assert-True ((Read-Snapshot).status -eq 'running') 'Cancel reset preserves active timer'
        Press-PetControl '提前结束并重置'
        [System.Windows.Forms.SendKeys]::SendWait('{ESC}')
        Start-Sleep -Milliseconds 300
        Assert-True ($null -eq (Find-PetControl '结束并重置')) 'Escape dismisses reset confirmation'
        Press-PetControl '提前结束并重置'
        Press-PetControl '结束并重置'
        Assert-True ((Read-Snapshot).status -eq 'ready') 'Confirmed reset returns timer to ready'
        Press-PetControl '和小番茄说话；按住拖动可移动'
        Assert-True ($null -ne (Find-PetControl '收到你的摸摸，今天也元气满满！')) 'Real pet click produces response'
        Press-PetControl '打开互动'
        [System.Windows.Forms.SendKeys]::SendWait('{ESC}')
        Start-Sleep -Milliseconds 300
        Assert-True ($null -eq (Find-PetControl '打招呼')) 'Escape closes interaction panel'
        function Save-PetImage([string]$name) {
            $r=New-Object TomatoSmoke+RECT
            [TomatoSmoke]::GetWindowRect($script:window,[ref]$r) | Out-Null
            $bitmap=New-Object System.Drawing.Bitmap ($r.Right-$r.Left),($r.Bottom-$r.Top)
            $graphics=[System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.CopyFromScreen($r.Left,$r.Top,0,0,$bitmap.Size)
            $bitmap.Save((Join-Path $evidenceRoot $name))
            $graphics.Dispose(); $bitmap.Dispose()
        }
        Start-Sleep -Seconds 3
        $before=New-Object TomatoSmoke+RECT
        [TomatoSmoke]::GetWindowRect($script:window,[ref]$before) | Out-Null
        [TomatoSmoke]::SetCursorPos(($before.Left-400),($before.Top+120)) | Out-Null
        Start-Sleep -Milliseconds 700
        Save-PetImage 'gaze-left.png'
        [TomatoSmoke]::SetCursorPos(($before.Right+8),($before.Top+120)) | Out-Null
        Start-Sleep -Milliseconds 700
        Save-PetImage 'gaze-right.png'
        $body=(Find-PetControl '和小番茄说话；按住拖动可移动').Current.BoundingRectangle
        $grabX=[int]($body.X+$body.Width/2); $grabY=[int]($body.Y+$body.Height/2)
        [TomatoSmoke]::SetCursorPos($grabX,$grabY) | Out-Null
        Start-Sleep -Milliseconds 250
        [TomatoSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        try {
            [TomatoSmoke]::SetCursorPos(($grabX-12),$grabY) | Out-Null
            Start-Sleep -Milliseconds 300
            for ($step=1; $step -le 8; $step++) {
                [TomatoSmoke]::SetCursorPos(($grabX-12-$step*22),($grabY-$step*8)) | Out-Null
                Start-Sleep -Milliseconds 60
            }
            Save-PetImage 'pet-lifted.png'
        } finally { [TomatoSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero) }
        Start-Sleep -Milliseconds 1200
        $after=New-Object TomatoSmoke+RECT
        [TomatoSmoke]::GetWindowRect($script:window,[ref]$after) | Out-Null
        Assert-True ($after.Left -lt $before.Left-100) 'Native pet drag moves actual window'
        Assert-True ((Read-Snapshot).status -eq 'ready') 'Dragging does not start or reset timer'
        Save-PetImage 'pet-landed.png'
        Press-PetControl '收起信息栏'
        Assert-True ($null -eq (Main-Control '计时器') -and $null -eq (Main-Control '打开互动')) 'Collapse button returns to pet only'
        Save-PetImage 'pet-only.png'
    }
    Write-Output "Evidence: $env:LITTLE_TOMATO_RUNTIME_DIR"
} finally {
    foreach ($owned in (@($duplicate,$first) + $pair)) {
        if ($null -ne $owned -and -not $owned.HasExited) { Stop-Process -Id $owned.Id }
    }
    Stop-Transcript | Out-Null
    if ($previousDpi -ne [IntPtr]::Zero) { [TomatoSmoke]::SetThreadDpiAwarenessContext($previousDpi) | Out-Null }
    $env:LITTLE_TOMATO_RUNTIME_DIR = $normalRuntime
}
