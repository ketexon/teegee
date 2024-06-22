using System;
using System.Threading.Tasks;

public static class AsyncUtil
{
    public static bool Wait(this Task task, TimeSpan timespan)
    {
        var t = WaitAsync(task, timespan);
        t.Wait();
        return t.Result;
    }

    public static bool Wait(this Task task, int msTimeout)
    {
        return task.Wait(TimeSpan.FromMilliseconds(msTimeout));
    }

    public static async Task<bool> WaitAsync(this Task task, TimeSpan timespan)
    {
        return await Task.WhenAny(task, Task.Delay(timespan)) == task;
    }


    public static Task<bool> WaitAsync(this Task task, int msTimeout)
    {
        return task.WaitAsync(TimeSpan.FromMilliseconds(msTimeout));
    }
}