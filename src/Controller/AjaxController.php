<?php

namespace App\Controller;

use App\Ampere\SystemInfo\OneSecond;
use App\Ampere\SystemInfo\ThreeSeconds;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\Routing\Annotation\Route;
use Symfony\Component\Serializer\SerializerInterface;

class AjaxController extends AbstractController
{
    #[Route('/one-second', name: 'oneSecond')]
    public function oneSecond(OneSecond $oneSecond, SerializerInterface $serializer): JsonResponse
    {
        return JsonResponse::fromJsonString($serializer->serialize($oneSecond->getDto(), 'json'));
    }

    #[Route('/three-second', name: 'threeSecond')]
    public function threeSecond(ThreeSeconds $threeSeconds, SerializerInterface $serializer): JsonResponse
    {
        return JsonResponse::fromJsonString($serializer->serialize($threeSeconds->getDto(), 'json'));
    }
}
