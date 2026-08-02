namespace CsBindings;

public class App1 : App
{
    public override void Initialize()
    {
        Console.WriteLine("App Init");

        var texturePath = Path.Combine(AppManager.ExecutingDirectory, "assets", "sprites", "cat.jpg");
        var texture = Texture.Load(texturePath);

        var canvas = AppManager.CreateCanvas();
        // var image = canvas.
        //     Image().
        //     SetTexture(texture).
        //     SetPosition(new Vector2(0, 0)).
        //     SetSize((Vector2)texture.Size);
        // var label = canvas.Label().SetText("ABCDEFG").SetFontSize(2000.0f).SetPosition(new Vector2(500, 500));
        var button = canvas.Button().AddOnClickListener(() =>
        {
            Console.WriteLine("clicked button");
        });
    }
}