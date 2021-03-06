<?php

namespace App\Ampere\DockerClient\Response;

class Parser
{
    public static function extractHeaders(string $content): array
    {
        $positionEndOfHeaders = self::getEndOfHeaders($content);
        if (!$positionEndOfHeaders) {
            return [];
        }
        $content = \substr($content, 0, $positionEndOfHeaders);
        $content = \explode("\r\n", $content);

        $headers = [];
        foreach ($content as $item) {
            $list = \explode(':', $item);
            if (1 === \count($list)) {
                continue;
            }

            $headers[$list[0]] = $list[1];
        }

        return $headers;
    }

    public static function extractContent(string $content, ?int $contentLength): string
    {
        if (!\is_null($contentLength)) {
            return self::cleanContent(\substr($content, -$contentLength));
        }
        $positionEndOfHeaders = self::getEndOfHeaders($content);
        if (!$positionEndOfHeaders) {
            $positionEndOfHeaders = 0;
        }

        $content = \substr($content, $positionEndOfHeaders, \strlen($content) - $positionEndOfHeaders);

        return self::cleanContent($content);
    }

    private static function getEndOfHeaders(string &$content): int|false
    {
        return \strpos($content, "\r\n\r\n");
    }

    private static function cleanContent(string $content): string
    {
        return \str_replace(["\r", "\n"], '', $content);
    }
}
