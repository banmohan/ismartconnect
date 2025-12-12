using System.Text.Json;
using System.Text.Json.Serialization;

namespace ISmartConnect.Helpers;

public static class JsonHelper
{
    private static readonly JsonSerializerOptions DefaultOptions = GetDefaultOptions();

    private static JsonSerializerOptions GetDefaultOptions() => new JsonSerializerOptions().AssignDefaultOptions();
    public static JsonSerializerOptions AssignDefaultOptions(this JsonSerializerOptions source)
    {
        source.ReferenceHandler = ReferenceHandler.IgnoreCycles;
        source.IncludeFields = true;
        source.PropertyNamingPolicy = JsonNamingPolicy.CamelCase;
        source.DictionaryKeyPolicy = JsonNamingPolicy.CamelCase;
        source.UnknownTypeHandling = JsonUnknownTypeHandling.JsonNode;
        source.Converters.Add(new DateOnlyJsonConverter());
        source.Converters.Add(new TimeOnlyJsonConverter());

        return source;
    }

    public static string Serialize<T>(T? obj) => JsonSerializer.Serialize(obj, DefaultOptions);
    public static T? Deserialize<T>(string json) => JsonSerializer.Deserialize<T>(json, DefaultOptions);
}