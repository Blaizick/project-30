using System.Diagnostics.CodeAnalysis;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Security.Cryptography.X509Certificates;
using System.Text;

namespace CsBindings;

[StructLayout(LayoutKind.Sequential)]
public unsafe partial struct UnmanagedCallbacks
{
    public delegate* unmanaged[Cdecl]<void> U_TestCall;
    public delegate* unmanaged[Cdecl]<ObjectIndex> U_CreateCanvas;
    public delegate* unmanaged[Cdecl]<ObjectIndex> U_CreateImage;
    public delegate* unmanaged[Cdecl]<NativeString, ObjectIndex> U_LoadTexture;
    public delegate* unmanaged[Cdecl]<ObjectIndex, ObjectIndex, void> U_SetTexture;
    public delegate* unmanaged[Cdecl]<NativeString> U_GetExecutingDirectory;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Vector2, void> U_SetPosition;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Vector2, void> U_SetSize;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Color, void> U_SetColor;
    public delegate* unmanaged[Cdecl]<ObjectIndex, ObjectIndex> U_GetTexture;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Vector2> U_GetPosition;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Vector2> U_GetSize;
    public delegate* unmanaged[Cdecl]<ObjectIndex, Color> U_GetColor;
    public delegate* unmanaged[Cdecl]<ObjectIndex, UVector2> U_GetTextureSize;
    public delegate* unmanaged[Cdecl]<ObjectIndex> U_CreateLabel;
    public delegate* unmanaged[Cdecl]<ObjectIndex, NativeString, void> U_SetText;
    public delegate* unmanaged[Cdecl]<ObjectIndex, NativeString> U_GetText;
    public delegate* unmanaged[Cdecl]<ObjectIndex, ObjectIndex, void> U_SetFont;
    public delegate* unmanaged[Cdecl]<ObjectIndex, ObjectIndex> U_GetFont;
    public delegate* unmanaged[Cdecl]<ObjectIndex, float, void> U_SetFontSize;
    public delegate* unmanaged[Cdecl]<ObjectIndex, float> U_GetFontSize;
    public delegate* unmanaged[Cdecl]<NativeString, ObjectIndex> U_LoadFont;
    public delegate* unmanaged[Cdecl]<Vector2> U_GetMousePosition;
    public delegate* unmanaged[Cdecl]<ObjectIndex> U_CreateButton;
    public delegate* unmanaged[Cdecl]<NativeString, IntPtr> U_GetFunctionPointer;
}
public delegate void U_TestCall2();

[StructLayout(LayoutKind.Sequential)]
public struct Vector2
{
    private float m_X;
    private float m_Y;

    public float X
    {
        get
        {
            return m_X;
        }
        set
        {
            m_X = value;
        }
    }

    public float Y
    {
        get
        {
            return m_Y;
        }
        set
        {
            m_Y = value;
        }
    }

    public Vector2()
    {
        
    }

    public Vector2(float x, float y)
    {
        m_X = x;
        m_Y = y;
    }
}

[StructLayout(LayoutKind.Sequential)]
public struct Color
{
    private float m_R;
    private float m_G;
    private float m_B;
    private float m_A;

    public float R
    {
        get
        {
            return m_R;
        }
        set
        {
            m_R = value;
        }
    }

    public float G
    {
        get
        {
            return m_G;
        }
        set
        {
            m_G = value;
        }
    }

    public float B
    {
        get
        {
            return m_B;
        }
        set
        {
            m_B = value;
        }
    }

    public float A
    {
        get
        {
            return m_A;
        }
        set
        {
            m_A = value;
        }
    }
}

[StructLayout(LayoutKind.Sequential)]
public unsafe struct ManagedCallbacks
{
    public delegate* unmanaged<void> M_TestCall;
    public delegate* unmanaged<ObjectIndex, void> M_OnButtonClicked;

    public static void Set(ref ManagedCallbacks managedCallbacks)
    {
        managedCallbacks.M_TestCall = &Engine.M_TestCall;
        managedCallbacks.M_OnButtonClicked = &OnButtonClicked;
    } 

    [UnmanagedCallersOnly]
    public static void OnButtonClicked(ObjectIndex objectIndex)
    {
        AppManager.EventBus.Fire(new ButtonClickedEvent()
        {
            objectIndex = objectIndex,
        });
    }
}

public class ButtonClickedEvent : IEvent
{
    public ObjectIndex objectIndex;
}

public interface IEvent
{
    
}

public class EventBus
{
    private Dictionary<Type, IQEvent> m_Listeners = new();

    public void AddListener<T>(Action<T> listener) where T : IEvent
    {
        var type = typeof(T);
        if (!m_Listeners.TryGetValue(type, out var _event))
        {
            _event = new QEvent<T>();
            m_Listeners[type] = _event;
        }
        ((QEvent<T>)_event).AddListener(listener);
    }

    public void Fire<T>(T eventArg) where T : IEvent
    {
        if (m_Listeners.TryGetValue(typeof(T), out var _event))
        {
            ((QEvent<T>)_event).Invoke(eventArg);
        }
    }
}

public interface IQEvent
{
    
}

public class QEvent<T> : IQEvent
{
    protected List<Action<T>> m_Listeners = new();

    public void Invoke(T arg)
    {
        foreach (var listener in m_Listeners)
        {
            listener.Invoke(arg);
        }
    }
    public void RemoveAllListeners()
    {
        m_Listeners.Clear();
    } 

    public void RemoveListener(Action<T> listener)
    {
        m_Listeners.Remove(listener);
    }

    public void AddListener(Action<T> listener)
    {
        m_Listeners.Add(listener);
    }
}

public class QEvent : IQEvent
{
    protected List<Action> m_Listeners = new();

    public void Invoke()
    {
        foreach (var listener in m_Listeners)
        {
            listener.Invoke();
        }
    }
    public void RemoveAllListeners()
    {
        m_Listeners.Clear();
    } 

    public void RemoveListener(Action listener)
    {
        m_Listeners.Remove(listener);
    }

    public void AddListener(Action listener)
    {
        m_Listeners.Add(listener);
    }
}


public interface ICachedManagedObjectsPool
{
    
} 

public class CachedManagedObjectsPool<T> : ICachedManagedObjectsPool where T : QObject
{
    public Dictionary<ObjectIndex, T> dic = new();

    public CachedObjectReturnContext<T> GetCachedObject(CachedObjectCallContext<T> callContext)
    {
        CachedObjectReturnContext<T> returnContext = new();
        if (!dic.TryGetValue(callContext.objectIndex, out returnContext._object))
        {
            returnContext.created = true;
            returnContext._object = Activator.CreateInstance<T>();
            var qObjectInstance = (QObject)returnContext._object;
            qObjectInstance.Set(callContext.objectIndex);
            dic[callContext.objectIndex] = returnContext._object;
            if (callContext.isUiElement)
            {
                returnContext.initialized = true;
                ((UiElement)qObjectInstance).Init();
            }
        }
        return returnContext;
    } 
}
public struct CachedObjectReturnContext<T> where T : QObject
{
    public bool initialized;
    public bool created;
    public T _object;
}

public struct CachedObjectCallContext<T> where T : QObject
{
    public ObjectIndex objectIndex;
    public bool isUiElement;

    public CachedObjectCallContext(ObjectIndex objectIndex, bool isUiElement = false)
    {
        this.objectIndex = objectIndex;
        this.isUiElement = isUiElement;
    }
}

public static class Engine
{
    public static UnmanagedCallbacks unmanagedCallbacks;
    public static Dictionary<Type, ICachedManagedObjectsPool> managedObjectPools = new();

    public static CachedObjectReturnContext<T> GetQObjectInstance<T>(CachedObjectCallContext<T> callContext) where T : QObject
    {
        var type = typeof(T);
        if (!managedObjectPools.TryGetValue(type, out var pool))
        {
            pool = new CachedManagedObjectsPool<T>();
            managedObjectPools[type] = pool;
        }
        var castedPool = (CachedManagedObjectsPool<T>)pool;
        return castedPool.GetCachedObject(callContext);
    }

    [UnmanagedCallersOnly]
    public static void M_TestCall()
    {
        Console.WriteLine("Managed Test Call");        
    }

    [UnmanagedCallersOnly]
    public static unsafe void InitializeFromEngine(UnmanagedCallbacks unmanagedCallbacks, 
        int unmanagedCallbacksSize, 
        ManagedCallbacks* managedCallbacksPtr, 
        int* managedCallbacksSizePtr)
    {
        if (sizeof(UnmanagedCallbacks) != unmanagedCallbacksSize)
        {
            throw new Exception("Unmanaged callbacks size mismatches");
        }

        Engine.unmanagedCallbacks = unmanagedCallbacks;

        ref var managedCallbacksRef = ref Unsafe.AsRef<ManagedCallbacks>(managedCallbacksPtr);
        ManagedCallbacks.Set(ref managedCallbacksRef);
        managedCallbacksSizePtr[0] = sizeof(ManagedCallbacks);
        
        Debug.GetCallBacks();

        if (AppManager.FindApp())
        {
            AppManager.App.Initialize();
        }
    }

    public static void GetProgramm()
    {
        
    }
}

public struct IdBindings
{
    
}

public struct ObjectIndex
{
    public uint id;

    public bool Alive
    {
        get
        {
            return false;
        }
    }

    public static bool operator ==(ObjectIndex left, ObjectIndex right)
    {
        return left.Equals(right);
    }

    public static bool operator !=(ObjectIndex left, ObjectIndex right)
    {
        return !left.Equals(right);
    }

    public override bool Equals([NotNullWhen(true)] object? obj)
    {
        if (obj is ObjectIndex other)
        {
            return other.id == id;
        }
        return false;
    }

    public override int GetHashCode()
    {
        return id.GetHashCode();
    }

    public override string ToString()
    {
        return $"Id: {id}";
    }
}

public class AppManager
{
    public unsafe static Canvas CreateCanvas()
    {
        return new Canvas(Engine.unmanagedCallbacks.U_CreateCanvas());
    }

    private static App s_App = null;
    public static App App
    {
        get
        {
            return s_App;
        }
    }

    public static bool FindApp()
    {
        Type appType = typeof(App);
        foreach (var assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            foreach (var type in assembly.GetTypes())
            {
                if (!appType.IsAssignableFrom(type))
                {
                    continue;
                }
                if (type.IsAbstract)
                {
                    continue;
                }
                s_App = (App)Activator.CreateInstance(type);
                return true;
            }
        }
        return false;
    }

    public static unsafe string ExecutingDirectory
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetExecutingDirectory().ToString();
        }
    }

    private static EventBus s_EventBus = new();

    public static EventBus EventBus
    {
        get
        {
            return s_EventBus;
        }
    }
}

public class QObject
{
    private ObjectIndex m_Id;

    public ObjectIndex Id
    {
        get
        {
            return m_Id;
        }
    }
    public QObject()
    {
        
    }

    public QObject(ObjectIndex id)
    {
        m_Id = id;
    }

    public void Set(ObjectIndex id)
    {
        m_Id = id;
    }
}

public abstract class App
{
    public abstract void Initialize();
}

public unsafe class Canvas : QObject
{
    public Canvas(ObjectIndex objectId) : base(objectId)
    {
        
    }

    public Button Button()
    {
        return Engine.GetQObjectInstance<Button>(new(Engine.unmanagedCallbacks.U_CreateButton(), true))._object;
    }
    public Image Image()
    {
        return Engine.GetQObjectInstance<Image>(new(Engine.unmanagedCallbacks.U_CreateImage(), true))._object;
    }
    public Label Label()
    {
        return Engine.GetQObjectInstance<Label>(new(Engine.unmanagedCallbacks.U_CreateLabel(), true))._object;
    }
}

public unsafe class UiElement : QObject
{
    public Vector2 Position
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetPosition(Id);
        }
        set
        {
            Engine.unmanagedCallbacks.U_SetPosition(Id, value);
        }
    }

    public Vector2 Size
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetSize(Id);
        }
        set
        {
            Engine.unmanagedCallbacks.U_SetSize(Id, value);
        }
    }
    
    public UiElement() : base() {}
    public UiElement(ObjectIndex objectIndex) : base(objectIndex) {}

    public UiElement SetPosition(Vector2 position)
    {
        Position = position;
        return this;
    }

    public UiElement SetSize(Vector2 size)
    {
        Size = size;
        return this;
    }

    public virtual void Init()
    {
        
    }
}

public class Button : UiElement
{
    public QEvent onClick = new();

    public Button() : base() {}
    public Button(ObjectIndex objectIndex) : base(objectIndex) {}

    public override void Init()
    {
        AppManager.EventBus.AddListener<ButtonClickedEvent>(ctx =>
        {
            if (ctx.objectIndex == Id)
            {
                onClick.Invoke();
            }
        });

        base.Init();
    }

    public Button AddOnClickListener(Action listener)
    {
        onClick.AddListener(listener);
        return this;
    }
}

public unsafe class Image : UiElement
{
    public Texture Texture
    {
        set
        {
            Engine.unmanagedCallbacks.U_SetTexture(Id, value.Id);            
        }
        get
        {
            return Engine.GetQObjectInstance<Texture>(new(Engine.unmanagedCallbacks.U_GetTexture(Id)))._object;
        }
    }

    public Image() : base() {}
    public Image(ObjectIndex objectIndex) : base(objectIndex) {}

    public Image SetTexture(Texture texture)
    {
        Texture = texture;
        return this;
    }
}

public unsafe class Texture : QObject
{
    public Texture() : base() {}
    public Texture(ObjectIndex objectIndex) : base(objectIndex) {}

    public UVector2 Size
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetTextureSize(Id);
        }
    }

    public static unsafe Texture Load(string path)
    {
        ObjectIndex objectIndex;
        using (NativeString nativeString = NativeString.FromString(path))
        {
            objectIndex = Engine.unmanagedCallbacks.U_LoadTexture(nativeString);
        }
        return Engine.GetQObjectInstance<Texture>(new(objectIndex))._object;
    }
}

[StructLayout(LayoutKind.Sequential)]
public unsafe struct NativeString : IDisposable
{
    public byte* ptr;
    public uint len;

    public static NativeString FromString(string _string)
    {
        byte[] bytes = Encoding.UTF8.GetBytes(_string);
        IntPtr mem = Marshal.AllocHGlobal(bytes.Length);
        Marshal.Copy(bytes, 0, mem, bytes.Length);
        return new NativeString()
        {
            ptr = (byte*)mem,
            len = (uint)bytes.Length,
        };
    }

    public override string ToString()
    {
        return Encoding.UTF8.GetString(ptr, (int)len);        
    }

    public void Free()
    {
        Marshal.FreeHGlobal((IntPtr)ptr);
    }

    public void Dispose()
    {
        Free();
    }
}

[StructLayout(LayoutKind.Sequential)]
public struct UVector2
{
    private uint m_X;
    private uint m_Y;

    public uint X
    {
        get
        {
            return m_X;
        }
        set
        {
            m_X = value;
        }
    }
    public uint Y
    {
        get
        {
            return m_Y;
        }
        set
        {
            m_Y = value;
        }
    }

    public UVector2() {}
    public UVector2(uint x, uint y)
    {
        m_X = x;
        m_Y = y;
    }

    public static implicit operator UVector2(Vector2 vec)
    {
        return new UVector2((uint)vec.X, (uint)vec.Y);
    }
    public static implicit operator Vector2(UVector2 vec)
    {
        return new Vector2((float)vec.X, (float)vec.Y);
    }
}

public unsafe class Label : UiElement
{
    public string Text
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetText(Id).ToString();
        }
        set
        {
            using (NativeString text = NativeString.FromString(value))
            {
                Engine.unmanagedCallbacks.U_SetText(Id, text);
            }
        }
    }
    public Font Font
    {
        get
        {
            var fontId = Engine.unmanagedCallbacks.U_GetFont(Id);
            return Engine.GetQObjectInstance<Font>(new(fontId))._object;
        }
        set
        {
            Engine.unmanagedCallbacks.U_SetFont(Id, value.Id);
        }
    }
    public float FontSize
    {
        get
        {
            return Engine.unmanagedCallbacks.U_GetFontSize(Id);
        }
        set
        {
            Engine.unmanagedCallbacks.U_SetFontSize(Id, value);
        }
    }

    public Label SetText(string text)
    {
        Text = text;
        return this;
    }
    
    public Label SetFont(Font font)
    {
        Font = font;
        return this;
    }
    
    public Label SetFontSize(float fontSize)
    {
        FontSize = fontSize;
        return this;
    }
}

public unsafe class Font : QObject
{
    public static Font Load(string path)
    {
        ObjectIndex objectIndex;
        using (NativeString nativeString = NativeString.FromString(path))
        {
            objectIndex = Engine.unmanagedCallbacks.U_LoadFont(nativeString);
        }
        return Engine.GetQObjectInstance<Font>(new(objectIndex))._object;
    }
}

public enum LogChannel : ulong
{
    Default = 1 << 0,
    Render = 1 << 1,
    Interop = 1 << 2,
    App = 1 << 3,
}

public static unsafe class Debug
{
    public struct Callbacks
    {
        public delegate* unmanaged<ulong, NativeString, void> U_Log;
        public delegate* unmanaged<ulong, NativeString, void> U_VerboseLog;
        public delegate* unmanaged<ulong, NativeString, void> U_Warning;
        public delegate* unmanaged<ulong, NativeString, void> U_VerboseWarning;
        public delegate* unmanaged<ulong, NativeString, void> U_Error;
        public delegate* unmanaged<ulong, NativeString, void> U_VerboseError;
    }

    public static Callbacks callbacks;

    public static void GetCallBacks()
    {
        callbacks = new();

        using (NativeString nativeFunctionName = NativeString.FromString("u_log"))
            callbacks.U_Log = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
        using (NativeString nativeFunctionName = NativeString.FromString("u_verbose_log"))
            callbacks.U_VerboseLog = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
        using (NativeString nativeFunctionName = NativeString.FromString("u_warning"))
            callbacks.U_Warning = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
        using (NativeString nativeFunctionName = NativeString.FromString("u_verbose_warning"))
            callbacks.U_VerboseWarning = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
        using (NativeString nativeFunctionName = NativeString.FromString("u_error"))
            callbacks.U_Error = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
        using (NativeString nativeFunctionName = NativeString.FromString("u_verbose_error"))
            callbacks.U_VerboseError = (delegate* unmanaged<ulong, NativeString, void>)Engine.unmanagedCallbacks.U_GetFunctionPointer(nativeFunctionName);
    }

    public static void Log(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            Log(_object.ToString(), channel);
        }
    }
    public static void Log(string message, LogChannel channel = LogChannel.Default)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_Log((ulong)channel, nativeMessage);
        }
    }
    public static void VerboseLog(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            VerboseLog(_object.ToString(), channel);
        }
    }
    public static void VerboseLog(LogChannel channel, string message)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_VerboseLog((ulong)channel, nativeMessage);
        }
    }
    public static void Warning(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            Warning(_object.ToString(), channel);
        }
    }
    public static void Warning(LogChannel channel, string message)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_Warning((ulong)channel, nativeMessage);
        }
    }
    public static void VerboseWarning(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            VerboseWarning(_object.ToString(), channel);
        }
    }
    public static void VerboseWarning(LogChannel channel, string message)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_VerboseWarning((ulong)channel, nativeMessage);
        }
    }
    public static void Error(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            Error(_object.ToString(), channel);
        }
    }
    public static void Error(LogChannel channel, string message)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_Error((ulong)channel, nativeMessage);
        }
    }
    public static void VerboseError(object _object, LogChannel channel = LogChannel.Default) 
    {
        if (_object != null)
        {
            VerboseError(_object.ToString(), channel);
        }
    }
    public static void VerboseError(LogChannel channel, string message)
    {
        using (NativeString nativeMessage = NativeString.FromString(message))
        {
            callbacks.U_VerboseError((ulong)channel, nativeMessage);
        }
    }
}