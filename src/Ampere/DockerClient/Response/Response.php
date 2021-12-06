<?php

namespace App\Ampere\DockerClient\Response;

use App\Ampere\DockerClient\Response\Exception\InvalidJsonException;

class Response
{
    private array $headers = [];
    private array $content = [];

    public function __construct(string $response)
    {
        $this->headers = Parser::extractHeaders($response);
        $body = Parser::extractContent(
            $response,
            $this->headers['Content-Length'] ?? null
        );

        if (0 === \strlen($body)) {
            $this->content = [];
        } else {
            if (!$this->validateContentAsJson($body)) {
                throw new InvalidJsonException('Content is not valid json.');
            }
            $this->content = \json_decode($body, true);
        }
    }

    private function validateContentAsJson($body): bool
    {
        return null !== \json_decode($body, true);
    }

    public function getHeaders(): array
    {
        return $this->headers;
    }

    public function getContent(): array
    {
        return $this->content;
    }
}
