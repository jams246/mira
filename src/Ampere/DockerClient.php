<?php

namespace App\Ampere;

use CurlHandle;

/**
 * Class DockerClient.
 *
 * @url https://gist.github.com/FergusInLondon/c3eb96f1f6565a81e509ece6b14b9a78
 */
class DockerClient
{
    private false|CurlHandle $curlClient;

    public function __construct()
    {
        $this->curlClient = \curl_init();

        \curl_setopt($this->curlClient, CURLOPT_UNIX_SOCKET_PATH, '/var/run/docker.sock');
        \curl_setopt($this->curlClient, CURLOPT_RETURNTRANSFER, true);
        \curl_setopt($this->curlClient, CURLOPT_HEADER, false);
        \curl_setopt($this->curlClient, CURLOPT_IPRESOLVE, CURL_IPRESOLVE_WHATEVER);
    }

    private function generateRequestUri(string $requestPath): string
    {
        return \sprintf('http://unixsocket%s', $requestPath);
    }

    public function dispatchCommand(string $endpoint): array
    {
        /* @phpstan-ignore-next-line */
        \curl_setopt($this->curlClient, CURLOPT_URL, $this->generateRequestUri($endpoint));

        /* @phpstan-ignore-next-line */
        $result = \curl_exec($this->curlClient);

        /* @phpstan-ignore-next-line */
        return \json_decode($result);
    }

    public function multiCurl(array $ids): array
    {
        $master = \curl_multi_init();

        $curl_arr = [];
        foreach ($ids as $key => $id) {
            $curl_arr[$key] = \curl_init();

            \curl_setopt($curl_arr[$key], CURLOPT_UNIX_SOCKET_PATH, '/var/run/docker.sock');
            \curl_setopt($curl_arr[$key], CURLOPT_RETURNTRANSFER, true);
            \curl_setopt($curl_arr[$key], CURLOPT_HEADER, false);
            \curl_setopt($curl_arr[$key], CURLOPT_IPRESOLVE, CURL_IPRESOLVE_WHATEVER);
            \curl_setopt($curl_arr[$key], CURLOPT_URL, $this->generateRequestUri($id));

            \curl_multi_add_handle($master, $curl_arr[$key]);
        }

        $active = false;
        do {
            $status = \curl_multi_exec($master, $active);
            if ($active) {
                \curl_multi_select($master);
            }
        } while ($active && CURLM_OK == $status);

        $results = [];
        foreach ($ids as $key => $id) {
            /* @phpstan-ignore-next-line */
            $results[] = \json_decode(\curl_multi_getcontent($curl_arr[$key]));

            \curl_multi_remove_handle($master, $curl_arr[$key]);
            \curl_close($curl_arr[$key]);
        }

        \curl_multi_close($master);

        return $results;
    }
}
