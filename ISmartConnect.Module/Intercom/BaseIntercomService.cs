using System.Net;
using System.Net.Http.Json;
using ISmartConnect.Module.Contracts;
using ISmartConnect.Module.Models;

namespace ISmartConnect.Module.Intercom;

public class BaseIntercomService
{
    public readonly HttpClient _httpClient;

    #region Handler

    private static Task DefaultHandlerDelegate(HttpResponseMessage responseMessage) =>
        responseMessage.StatusCode switch
        {
            HttpStatusCode.OK => Task.CompletedTask,
            HttpStatusCode.BadGateway => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Continue => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.SwitchingProtocols => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Processing => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.EarlyHints => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Created => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Accepted => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NonAuthoritativeInformation => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NoContent => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.ResetContent => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.PartialContent => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.MultiStatus => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.AlreadyReported => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.IMUsed => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Ambiguous => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Moved => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Found => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RedirectMethod => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NotModified => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.UseProxy => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Unused => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RedirectKeepVerb => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.PermanentRedirect => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.BadRequest => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Unauthorized => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.PaymentRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Forbidden => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NotFound => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.MethodNotAllowed => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NotAcceptable => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.ProxyAuthenticationRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RequestTimeout => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Conflict => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Gone => throw new MicroserviceRequestException((int)responseMessage.StatusCode,
                responseMessage.ReasonPhrase),
            HttpStatusCode.LengthRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.PreconditionFailed => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RequestEntityTooLarge => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RequestUriTooLong => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.UnsupportedMediaType => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RequestedRangeNotSatisfiable => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.ExpectationFailed => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.MisdirectedRequest => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.UnprocessableEntity => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.Locked => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.FailedDependency => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.UpgradeRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.PreconditionRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.TooManyRequests => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.RequestHeaderFieldsTooLarge => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.UnavailableForLegalReasons => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.InternalServerError => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NotImplemented => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.ServiceUnavailable => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.GatewayTimeout => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.HttpVersionNotSupported => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.VariantAlsoNegotiates => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.InsufficientStorage => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.LoopDetected => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NotExtended => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            HttpStatusCode.NetworkAuthenticationRequired => throw new MicroserviceRequestException(
                (int)responseMessage.StatusCode, responseMessage.ReasonPhrase),
            _ => throw new Exception("Unknown Error")
        };

    private static async Task<T> DefaultHandlerDelegate<T>(HttpResponseMessage responseMessage)
    {
        var error = await responseMessage.Content.ReadFromJsonAsync<ErrorModel>();
        switch (responseMessage.StatusCode)
        {
            case HttpStatusCode.OK:
                var model = await responseMessage.Content.ReadFromJsonAsync<T>();
                return model ?? throw new InvalidOperationException("Cannot parse the json value to model");
            case HttpStatusCode.InternalServerError:
                throw new MicroserviceRequestException(500, error?.Error);
            case HttpStatusCode.BadGateway:
                throw new MicroserviceRequestException(502, responseMessage.ReasonPhrase);
            case HttpStatusCode.BadRequest:
                throw new MicroserviceRequestException(400, error?.Error);
            case HttpStatusCode.Unauthorized:
                throw new MicroserviceRequestException(401, "Unauthorized");
            case HttpStatusCode.Continue:
            case HttpStatusCode.SwitchingProtocols:
            case HttpStatusCode.Processing:
            case HttpStatusCode.EarlyHints:
            case HttpStatusCode.Created:
            case HttpStatusCode.Accepted:
            case HttpStatusCode.NonAuthoritativeInformation:
            case HttpStatusCode.NoContent:
                return default(T)!;
            case HttpStatusCode.ResetContent:
            case HttpStatusCode.PartialContent:
            case HttpStatusCode.MultiStatus:
            case HttpStatusCode.AlreadyReported:
            case HttpStatusCode.IMUsed:
            case HttpStatusCode.Ambiguous:
            case HttpStatusCode.Moved:
            case HttpStatusCode.Found:
            case HttpStatusCode.RedirectMethod:
            case HttpStatusCode.NotModified:
            case HttpStatusCode.UseProxy:
            case HttpStatusCode.Unused:
            case HttpStatusCode.RedirectKeepVerb:
            case HttpStatusCode.PermanentRedirect:
            case HttpStatusCode.PaymentRequired:
            case HttpStatusCode.Forbidden:
            case HttpStatusCode.NotFound:
                throw new MicroserviceRequestException(404, "content not found");
            case HttpStatusCode.MethodNotAllowed:
            case HttpStatusCode.NotAcceptable:
            case HttpStatusCode.ProxyAuthenticationRequired:
            case HttpStatusCode.RequestTimeout:
            case HttpStatusCode.Conflict:
            case HttpStatusCode.Gone:
            case HttpStatusCode.LengthRequired:
            case HttpStatusCode.PreconditionFailed:
            case HttpStatusCode.RequestEntityTooLarge:
            case HttpStatusCode.RequestUriTooLong:
            case HttpStatusCode.UnsupportedMediaType:
            case HttpStatusCode.RequestedRangeNotSatisfiable:
            case HttpStatusCode.ExpectationFailed:
            case HttpStatusCode.MisdirectedRequest:
            case HttpStatusCode.UnprocessableEntity:
            case HttpStatusCode.Locked:
            case HttpStatusCode.FailedDependency:
            case HttpStatusCode.UpgradeRequired:
            case HttpStatusCode.PreconditionRequired:
            case HttpStatusCode.TooManyRequests:
            case HttpStatusCode.RequestHeaderFieldsTooLarge:
            case HttpStatusCode.UnavailableForLegalReasons:
            case HttpStatusCode.NotImplemented:
            case HttpStatusCode.ServiceUnavailable:
            case HttpStatusCode.GatewayTimeout:
            case HttpStatusCode.HttpVersionNotSupported:
            case HttpStatusCode.VariantAlsoNegotiates:
            case HttpStatusCode.InsufficientStorage:
            case HttpStatusCode.LoopDetected:
            case HttpStatusCode.NotExtended:
            case HttpStatusCode.NetworkAuthenticationRequired:
            default:
                throw new ArgumentOutOfRangeException();
        }
    }

    #endregion

    protected BaseIntercomService(MicroService microService, IMicroServiceMeta microServiceMeta, IUserMeta meta)
    {
        _httpClient = new HttpClient();
        _httpClient.BaseAddress = new Uri(microServiceMeta.GetUrl(microService));
        _httpClient.DefaultRequestHeaders.Add("x-api-key", microServiceMeta.ApiKey);
        _httpClient.DefaultRequestHeaders.Add("tenant", meta.ClientCode);
    }

    public async Task<T> Get<T>(string endpoint, Dictionary<string, string>? queryParams = null,
        Task<T>? messageHandler = null) where T : new()
    {
        queryParams ??= new Dictionary<string, string>();
        var queryString = string.Join("&", queryParams.Select(kvp => $"{kvp.Key}={kvp.Value}"));
        var url = string.IsNullOrWhiteSpace(queryString) ? endpoint : $"{endpoint}?{queryString}";
        var response = await _httpClient.GetAsync(url);
        messageHandler ??= DefaultHandlerDelegate<T>(response);
        return await messageHandler;
    }

    public async Task<ICollection<T>> GetMany<T>(string endpoint, Dictionary<string, string>? queryParams = null,
        Task<ICollection<T>>? messageHandler = null)
    {
        queryParams ??= new Dictionary<string, string>();
        var queryString = string.Join("&", queryParams.Select(kvp => $"{kvp.Key}={kvp.Value}"));
        var url = string.IsNullOrWhiteSpace(queryString) ? endpoint : $"{endpoint}?{queryString}";
        var response = await _httpClient.GetAsync(url);
        messageHandler ??= DefaultHandlerDelegate<ICollection<T>>(response);
        return await messageHandler;
    }

    public async Task<TResponse> Post<TResponse, TRequest>(string endpoint, TRequest requestBody,
        Task<TResponse>? messageHandler = null) where TResponse : new()
    {
        var response = await _httpClient.PostAsJsonAsync(endpoint, requestBody);
        messageHandler ??= DefaultHandlerDelegate<TResponse>(response);
        return await messageHandler;
    }

    protected async Task Post(string endpoint, object requestBody, Task? messageHandler = null)
    {
        var response = await _httpClient.PostAsJsonAsync(endpoint, requestBody);
        messageHandler ??= DefaultHandlerDelegate(response);
        await messageHandler;
    }

    public async Task<T> Put<T, TT>(string endpoint, TT requestBody, Task<T>? messageHandler = null) where T : new()
    {
        var response = await _httpClient.PutAsJsonAsync(endpoint, requestBody);
        messageHandler ??= DefaultHandlerDelegate<T>(response);
        return await messageHandler;
    }

    public async Task Put(string endpoint, object requestBody, Task? messageHandler = null)
    {
        var response = await _httpClient.PutAsJsonAsync(endpoint, requestBody);
        messageHandler ??= DefaultHandlerDelegate(response);
        await messageHandler;
    }

    public async Task Delete(string endpoint, Task? messageHandler = null)
    {
        var response = await _httpClient.DeleteAsync(endpoint);
        messageHandler ??= DefaultHandlerDelegate(response);
        await messageHandler;
    }

    private class MicroserviceRequestException : Exception
    {
        public MicroserviceRequestException(int statusCode, string? message) : base(
            message ?? "An error occurred while calling a microservice.")
        {
        }
    }
}