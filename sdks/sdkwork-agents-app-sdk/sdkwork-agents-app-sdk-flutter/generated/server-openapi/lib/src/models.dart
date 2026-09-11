Map<String, dynamic>? _sdkworkAsMap(dynamic value) {
  if (value is Map<String, dynamic>) {
    return value;
  }
  if (value is Map) {
    return value.map((key, item) => MapEntry(key.toString(), item));
  }
  return null;
}

List<dynamic>? _sdkworkAsList(dynamic value) {
  return value is List ? value : null;
}

class MediaToolDirectoryResponse {
  final int code;
  final dynamic data;
  final String traceId;

  MediaToolDirectoryResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory MediaToolDirectoryResponse.fromJson(Map<String, dynamic> json) {
    return MediaToolDirectoryResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('MediaToolDirectoryResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('MediaToolDirectoryResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class MediaToolInvokeApiResponse {
  final int code;
  final dynamic data;
  final String traceId;

  MediaToolInvokeApiResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory MediaToolInvokeApiResponse.fromJson(Map<String, dynamic> json) {
    return MediaToolInvokeApiResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('MediaToolInvokeApiResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('MediaToolInvokeApiResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ToolAssetListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ToolAssetListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ToolAssetListResponse.fromJson(Map<String, dynamic> json) {
    return ToolAssetListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ToolAssetListResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class SdkWorkApiResponse {
  final int code;
  final dynamic data;
  final String traceId;

  SdkWorkApiResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory SdkWorkApiResponse.fromJson(Map<String, dynamic> json) {
    return SdkWorkApiResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('SdkWorkApiResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('SdkWorkApiResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class SdkWorkResourceData {
  final dynamic item;

  SdkWorkResourceData({
    required this.item
  });

  factory SdkWorkResourceData.fromJson(Map<String, dynamic> json) {
    return SdkWorkResourceData(
      item: json['item']
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'item': item,
    };
  }
}

class SdkWorkPageData {
  final List<dynamic> items;
  final PageInfo pageInfo;

  SdkWorkPageData({
    required this.items,
    required this.pageInfo
  });

  factory SdkWorkPageData.fromJson(Map<String, dynamic> json) {
    return SdkWorkPageData(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          throw FormatException('SdkWorkPageData.items is required');
        }
        return list
            .map((item) => item)
            .whereType<dynamic>()
            .toList();
      })(),
      pageInfo: (() {
        final map = _sdkworkAsMap(json['pageInfo']);
        if (map == null) {
          throw FormatException('SdkWorkPageData.pageInfo is required');
        }
        return PageInfo.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items.map((item) => item).toList(),
      'pageInfo': pageInfo.toJson(),
    };
  }
}

class PageInfo {
  final String mode;
  final int? page;
  final int? pageSize;
  final String? totalItems;
  final int? totalPages;
  final String? nextCursor;
  final bool? hasMore;

  PageInfo({
    required this.mode,
    this.page,
    this.pageSize,
    this.totalItems,
    this.totalPages,
    this.nextCursor,
    this.hasMore
  });

  factory PageInfo.fromJson(Map<String, dynamic> json) {
    return PageInfo(
      mode: (() {
        final value = json['mode']?.toString();
        if (value == null) {
          throw FormatException('PageInfo.mode is required');
        }
        return value;
      })(),
      page: json['page'] is int ? json['page'] : null,
      pageSize: json['pageSize'] is int ? json['pageSize'] : null,
      totalItems: json['totalItems']?.toString(),
      totalPages: json['totalPages'] is int ? json['totalPages'] : null,
      nextCursor: json['nextCursor']?.toString(),
      hasMore: json['hasMore'] is bool ? json['hasMore'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'mode': mode,
      'page': page,
      'pageSize': pageSize,
      'totalItems': totalItems,
      'totalPages': totalPages,
      'nextCursor': nextCursor,
      'hasMore': hasMore,
    };
  }
}

class AgentRecord {
  final String id;
  final String agentId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic> manifest;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String implementationType;
  final String status;
  final String visibility;
  final List<String> tags;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentRecord({
    required this.id,
    required this.agentId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.code,
    required this.displayName,
    this.description,
    required this.manifest,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    required this.implementationType,
    required this.status,
    required this.visibility,
    required this.tags,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentRecord.fromJson(Map<String, dynamic> json) {
    return AgentRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.id is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.agentId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.ownerUserId is required');
        }
        return value;
      })(),
      code: (() {
        final value = json['code']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.code is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.displayName is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      manifest: (() {
        final map = _sdkworkAsMap(json['manifest']);
        if (map == null) {
          throw FormatException('AgentRecord.manifest is required');
        }
        return map;
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: (() {
        final value = json['implementationType']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.implementationType is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.status is required');
        }
        return value;
      })(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.visibility is required');
        }
        return value;
      })(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          throw FormatException('AgentRecord.tags is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'agentId': agentId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'code': code,
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'status': status,
      'visibility': visibility,
      'tags': tags.map((item) => item).toList(),
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentManagementProfile {
  final String? author;
  final String? avatar;
  final String? categoryId;
  final String? color;
  final bool? debugMode;
  final String? iconName;
  final bool? jsonMode;
  final List<String>? knowledgeBaseIds;
  final bool? memoryEnabled;
  final String? model;
  final List<String>? skillIds;
  final List<String>? suggestedPrompts;
  final String? systemPrompt;
  final double? temperature;
  final List<String>? toolIds;
  final String? type;
  final String? users;
  final List<String>? voiceIds;
  final String? welcomeMessage;

  AgentManagementProfile({
    this.author,
    this.avatar,
    this.categoryId,
    this.color,
    this.debugMode,
    this.iconName,
    this.jsonMode,
    this.knowledgeBaseIds,
    this.memoryEnabled,
    this.model,
    this.skillIds,
    this.suggestedPrompts,
    this.systemPrompt,
    this.temperature,
    this.toolIds,
    this.type,
    this.users,
    this.voiceIds,
    this.welcomeMessage
  });

  factory AgentManagementProfile.fromJson(Map<String, dynamic> json) {
    return AgentManagementProfile(
      author: json['author']?.toString(),
      avatar: json['avatar']?.toString(),
      categoryId: json['categoryId']?.toString(),
      color: json['color']?.toString(),
      debugMode: json['debugMode'] is bool ? json['debugMode'] : null,
      iconName: json['iconName']?.toString(),
      jsonMode: json['jsonMode'] is bool ? json['jsonMode'] : null,
      knowledgeBaseIds: (() {
        final list = _sdkworkAsList(json['knowledgeBaseIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      memoryEnabled: json['memoryEnabled'] is bool ? json['memoryEnabled'] : null,
      model: json['model']?.toString(),
      skillIds: (() {
        final list = _sdkworkAsList(json['skillIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      suggestedPrompts: (() {
        final list = _sdkworkAsList(json['suggestedPrompts']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      temperature: json['temperature'] is num ? json['temperature'].toDouble() : null,
      toolIds: (() {
        final list = _sdkworkAsList(json['toolIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      type: json['type']?.toString(),
      users: json['users']?.toString(),
      voiceIds: (() {
        final list = _sdkworkAsList(json['voiceIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      welcomeMessage: json['welcomeMessage']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'author': author,
      'avatar': avatar,
      'categoryId': categoryId,
      'color': color,
      'debugMode': debugMode,
      'iconName': iconName,
      'jsonMode': jsonMode,
      'knowledgeBaseIds': knowledgeBaseIds?.map((item) => item).toList(),
      'memoryEnabled': memoryEnabled,
      'model': model,
      'skillIds': skillIds?.map((item) => item).toList(),
      'suggestedPrompts': suggestedPrompts?.map((item) => item).toList(),
      'systemPrompt': systemPrompt,
      'temperature': temperature,
      'toolIds': toolIds?.map((item) => item).toList(),
      'type': type,
      'users': users,
      'voiceIds': voiceIds?.map((item) => item).toList(),
      'welcomeMessage': welcomeMessage,
    };
  }
}

class AgentResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResponse.fromJson(Map<String, dynamic> json) {
    return AgentResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentListResponse.fromJson(Map<String, dynamic> json) {
    return AgentListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProviderBindingRecord {
  final String tenantId;
  final String agentId;
  final String bindingId;
  final String providerId;
  final String implementationKind;
  final String configurationProfileId;
  final List<String> capabilities;
  final bool active;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentProviderBindingRecord({
    required this.tenantId,
    required this.agentId,
    required this.bindingId,
    required this.providerId,
    required this.implementationKind,
    required this.configurationProfileId,
    required this.capabilities,
    required this.active,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentProviderBindingRecord.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.tenantId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.agentId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.bindingId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.providerId is required');
        }
        return value;
      })(),
      implementationKind: (() {
        final value = json['implementationKind']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.implementationKind is required');
        }
        return value;
      })(),
      configurationProfileId: (() {
        final value = json['configurationProfileId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.configurationProfileId is required');
        }
        return value;
      })(),
      capabilities: (() {
        final list = _sdkworkAsList(json['capabilities']);
        if (list == null) {
          throw FormatException('AgentProviderBindingRecord.capabilities is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      active: (() {
        final value = json['active'];
        if (value is! bool) {
          throw FormatException('AgentProviderBindingRecord.active is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'agentId': agentId,
      'bindingId': bindingId,
      'providerId': providerId,
      'implementationKind': implementationKind,
      'configurationProfileId': configurationProfileId,
      'capabilities': capabilities.map((item) => item).toList(),
      'active': active,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentProviderBindingResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProviderBindingResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProviderBindingResponse.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProviderBindingResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProviderBindingResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProviderBindingListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProviderBindingListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProviderBindingListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProviderBindingListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProviderBindingListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentRuntimeExecutionRecord {
  final String tenantId;
  final String agentId;
  final String executionId;
  final String operation;
  final String status;
  final Map<String, dynamic> inputPayload;
  final Map<String, dynamic> outputPayload;
  final String requestedAt;
  final String completedAt;

  AgentRuntimeExecutionRecord({
    required this.tenantId,
    required this.agentId,
    required this.executionId,
    required this.operation,
    required this.status,
    required this.inputPayload,
    required this.outputPayload,
    required this.requestedAt,
    required this.completedAt
  });

  factory AgentRuntimeExecutionRecord.fromJson(Map<String, dynamic> json) {
    return AgentRuntimeExecutionRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.tenantId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.agentId is required');
        }
        return value;
      })(),
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.executionId is required');
        }
        return value;
      })(),
      operation: (() {
        final value = json['operation']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.operation is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.status is required');
        }
        return value;
      })(),
      inputPayload: (() {
        final map = _sdkworkAsMap(json['inputPayload']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionRecord.inputPayload is required');
        }
        return map;
      })(),
      outputPayload: (() {
        final map = _sdkworkAsMap(json['outputPayload']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionRecord.outputPayload is required');
        }
        return map;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.requestedAt is required');
        }
        return value;
      })(),
      completedAt: (() {
        final value = json['completedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.completedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'agentId': agentId,
      'executionId': executionId,
      'operation': operation,
      'status': status,
      'inputPayload': inputPayload,
      'outputPayload': outputPayload,
      'requestedAt': requestedAt,
      'completedAt': completedAt,
    };
  }
}

class AgentRuntimeExecutionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentRuntimeExecutionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentRuntimeExecutionResponse.fromJson(Map<String, dynamic> json) {
    return AgentRuntimeExecutionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentRuntimeExecutionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentCompositionSlotRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String agentId;
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int priority;
  final bool enabled;
  final String policyJson;
  final String status;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentCompositionSlotRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.agentId,
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.priority,
    required this.enabled,
    required this.policyJson,
    required this.status,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentCompositionSlotRecord.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.organizationId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.agentId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotRecord.priority is required');
        }
        return value;
      })(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('AgentCompositionSlotRecord.enabled is required');
        }
        return value;
      })(),
      policyJson: (() {
        final value = json['policyJson']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.policyJson is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.status is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'agentId': agentId,
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'status': status,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentCompositionSlotResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentCompositionSlotResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentCompositionSlotResponse.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentCompositionSlotResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentCompositionSlotListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentCompositionSlotListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentCompositionSlotListResponse.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentCompositionSlotListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentCompositionSlotRequest {
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;
  final String requestedAt;

  CreateAgentCompositionSlotRequest({
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson,
    required this.requestedAt
  });

  factory CreateAgentCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentCompositionSlotRequest(
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentCompositionSlotRequest {
  final String? expectedVersion;
  final String? slotKind;
  final String? targetModule;
  final String? targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;
  final String requestedAt;

  UpdateAgentCompositionSlotRequest({
    this.expectedVersion,
    this.slotKind,
    this.targetModule,
    this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson,
    required this.requestedAt
  });

  factory UpdateAgentCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentCompositionSlotRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      slotKind: json['slotKind']?.toString(),
      targetModule: json['targetModule']?.toString(),
      targetRef: json['targetRef']?.toString(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentCompositionSlotRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentRequest {
  final String agentId;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic> manifest;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String? implementationType;
  final String visibility;
  final List<String>? tags;
  final String requestedAt;

  CreateAgentRequest({
    required this.agentId,
    required this.code,
    required this.displayName,
    this.description,
    required this.manifest,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    this.implementationType,
    required this.visibility,
    this.tags,
    required this.requestedAt
  });

  factory CreateAgentRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentRequest(
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.agentId is required');
        }
        return value;
      })(),
      code: (() {
        final value = json['code']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.code is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.displayName is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      manifest: (() {
        final map = _sdkworkAsMap(json['manifest']);
        if (map == null) {
          throw FormatException('CreateAgentRequest.manifest is required');
        }
        return map;
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: json['implementationType']?.toString(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.visibility is required');
        }
        return value;
      })(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentId': agentId,
      'code': code,
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'visibility': visibility,
      'tags': tags?.map((item) => item).toList(),
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentProviderBindingRequest {
  final String bindingId;
  final String providerId;
  final String implementationKind;
  final String configurationProfileId;
  final List<String>? capabilities;
  final bool? makeDefault;
  final String requestedAt;

  CreateAgentProviderBindingRequest({
    required this.bindingId,
    required this.providerId,
    required this.implementationKind,
    required this.configurationProfileId,
    this.capabilities,
    this.makeDefault,
    required this.requestedAt
  });

  factory CreateAgentProviderBindingRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProviderBindingRequest(
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.bindingId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.providerId is required');
        }
        return value;
      })(),
      implementationKind: (() {
        final value = json['implementationKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.implementationKind is required');
        }
        return value;
      })(),
      configurationProfileId: (() {
        final value = json['configurationProfileId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.configurationProfileId is required');
        }
        return value;
      })(),
      capabilities: (() {
        final list = _sdkworkAsList(json['capabilities']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      makeDefault: json['makeDefault'] is bool ? json['makeDefault'] : null,
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'bindingId': bindingId,
      'providerId': providerId,
      'implementationKind': implementationKind,
      'configurationProfileId': configurationProfileId,
      'capabilities': capabilities?.map((item) => item).toList(),
      'makeDefault': makeDefault,
      'requestedAt': requestedAt,
    };
  }
}

class ActivateAgentProviderBindingRequest {
  final String requestedAt;

  ActivateAgentProviderBindingRequest({
    required this.requestedAt
  });

  factory ActivateAgentProviderBindingRequest.fromJson(Map<String, dynamic> json) {
    return ActivateAgentProviderBindingRequest(
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ActivateAgentProviderBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentPreviewResponseRequest {
  final String executionId;
  final String content;
  final bool? debugMode;
  final bool? memoryEnabled;
  final String? model;
  final double? temperature;
  final Map<String, dynamic>? inputPayload;
  final String requestedAt;

  CreateAgentPreviewResponseRequest({
    required this.executionId,
    required this.content,
    this.debugMode,
    this.memoryEnabled,
    this.model,
    this.temperature,
    this.inputPayload,
    required this.requestedAt
  });

  factory CreateAgentPreviewResponseRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentPreviewResponseRequest(
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.executionId is required');
        }
        return value;
      })(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.content is required');
        }
        return value;
      })(),
      debugMode: json['debugMode'] is bool ? json['debugMode'] : null,
      memoryEnabled: json['memoryEnabled'] is bool ? json['memoryEnabled'] : null,
      model: json['model']?.toString(),
      temperature: json['temperature'] is num ? json['temperature'].toDouble() : null,
      inputPayload: _sdkworkAsMap(json['inputPayload']),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'executionId': executionId,
      'content': content,
      'debugMode': debugMode,
      'memoryEnabled': memoryEnabled,
      'model': model,
      'temperature': temperature,
      'inputPayload': inputPayload,
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentPromptOptimizationRequest {
  final String executionId;
  final String prompt;
  final Map<String, dynamic>? inputPayload;
  final String requestedAt;

  CreateAgentPromptOptimizationRequest({
    required this.executionId,
    required this.prompt,
    this.inputPayload,
    required this.requestedAt
  });

  factory CreateAgentPromptOptimizationRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentPromptOptimizationRequest(
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.executionId is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.prompt is required');
        }
        return value;
      })(),
      inputPayload: _sdkworkAsMap(json['inputPayload']),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'executionId': executionId,
      'prompt': prompt,
      'inputPayload': inputPayload,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentRequest {
  final String? displayName;
  final String? description;
  final Map<String, dynamic>? manifest;
  final String? visibility;
  final List<String>? tags;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String? implementationType;
  final String? expectedVersion;
  final String requestedAt;

  UpdateAgentRequest({
    this.displayName,
    this.description,
    this.manifest,
    this.visibility,
    this.tags,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    this.implementationType,
    this.expectedVersion,
    required this.requestedAt
  });

  factory UpdateAgentRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentRequest(
      displayName: json['displayName']?.toString(),
      description: json['description']?.toString(),
      manifest: _sdkworkAsMap(json['manifest']),
      visibility: json['visibility']?.toString(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: json['implementationType']?.toString(),
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'visibility': visibility,
      'tags': tags?.map((item) => item).toList(),
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class RestoreAgentRequest {
  final String? expectedVersion;
  final String requestedAt;

  RestoreAgentRequest({
    this.expectedVersion,
    required this.requestedAt
  });

  factory RestoreAgentRequest.fromJson(Map<String, dynamic> json) {
    return RestoreAgentRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentWorkspaceRecord {
  final String id;
  final String workspaceId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String name;
  final String? description;
  final bool isDefault;
  final String status;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? archivedAt;

  AgentWorkspaceRecord({
    required this.id,
    required this.workspaceId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.name,
    this.description,
    required this.isDefault,
    required this.status,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.archivedAt
  });

  factory AgentWorkspaceRecord.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.id is required');
        }
        return value;
      })(),
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.workspaceId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.ownerUserId is required');
        }
        return value;
      })(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      isDefault: (() {
        final value = json['isDefault'];
        if (value is! bool) {
          throw FormatException('AgentWorkspaceRecord.isDefault is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.status is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.updatedAt is required');
        }
        return value;
      })(),
      archivedAt: json['archivedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'workspaceId': workspaceId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'name': name,
      'description': description,
      'isDefault': isDefault,
      'status': status,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'archivedAt': archivedAt,
    };
  }
}

class AgentWorkspaceResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentWorkspaceResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentWorkspaceResponse.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentWorkspaceResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentWorkspaceResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentWorkspaceListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentWorkspaceListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentWorkspaceListResponse.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentWorkspaceListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentWorkspaceListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class EnsureDefaultAgentWorkspaceRequest {
  final String? name;

  EnsureDefaultAgentWorkspaceRequest({
    this.name
  });

  factory EnsureDefaultAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return EnsureDefaultAgentWorkspaceRequest(
      name: json['name']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
    };
  }
}

class CreateAgentWorkspaceRequest {
  final String name;
  final String? description;

  CreateAgentWorkspaceRequest({
    required this.name,
    this.description
  });

  factory CreateAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentWorkspaceRequest(
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentWorkspaceRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
      'description': description,
    };
  }
}

class UpdateAgentWorkspaceRequest {
  final String expectedVersion;
  final String? name;
  final String? description;

  UpdateAgentWorkspaceRequest({
    required this.expectedVersion,
    this.name,
    this.description
  });

  factory UpdateAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentWorkspaceRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentWorkspaceRequest.expectedVersion is required');
        }
        return value;
      })(),
      name: json['name']?.toString(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'name': name,
      'description': description,
    };
  }
}

class AgentWorkspaceMutationRequest {
  final String expectedVersion;

  AgentWorkspaceMutationRequest({
    required this.expectedVersion
  });

  factory AgentWorkspaceMutationRequest.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceMutationRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceMutationRequest.expectedVersion is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
    };
  }
}

class AgentProjectRecord {
  final String id;
  final String projectId;
  final String workspaceId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String name;
  final String? description;
  final String visibility;
  final String status;
  final String driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;
  final String? importSourceKind;
  final String? importSourceRef;
  final String? driveSpaceId;
  final String? driveRootEntryId;
  final String? driveLogicalPath;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? archivedAt;

  AgentProjectRecord({
    required this.id,
    required this.projectId,
    required this.workspaceId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.name,
    this.description,
    required this.visibility,
    required this.status,
    required this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId,
    this.importSourceKind,
    this.importSourceRef,
    this.driveSpaceId,
    this.driveRootEntryId,
    this.driveLogicalPath,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.archivedAt
  });

  factory AgentProjectRecord.fromJson(Map<String, dynamic> json) {
    return AgentProjectRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.id is required');
        }
        return value;
      })(),
      projectId: (() {
        final value = json['projectId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.projectId is required');
        }
        return value;
      })(),
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.workspaceId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.ownerUserId is required');
        }
        return value;
      })(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.visibility is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.status is required');
        }
        return value;
      })(),
      driveAccessMode: (() {
        final value = json['driveAccessMode']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.driveAccessMode is required');
        }
        return value;
      })(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString(),
      importSourceKind: json['importSourceKind']?.toString(),
      importSourceRef: json['importSourceRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveRootEntryId: json['driveRootEntryId']?.toString(),
      driveLogicalPath: json['driveLogicalPath']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.updatedAt is required');
        }
        return value;
      })(),
      archivedAt: json['archivedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'projectId': projectId,
      'workspaceId': workspaceId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'name': name,
      'description': description,
      'visibility': visibility,
      'status': status,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
      'importSourceKind': importSourceKind,
      'importSourceRef': importSourceRef,
      'driveSpaceId': driveSpaceId,
      'driveRootEntryId': driveRootEntryId,
      'driveLogicalPath': driveLogicalPath,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'archivedAt': archivedAt,
    };
  }
}

class AgentProjectResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProjectListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentProjectRequest {
  final String? projectId;
  final String? workspaceId;
  final String name;
  final String? description;
  final String? visibility;
  final String? driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;

  CreateAgentProjectRequest({
    this.projectId,
    this.workspaceId,
    required this.name,
    this.description,
    this.visibility,
    this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId
  });

  factory CreateAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProjectRequest(
      projectId: json['projectId']?.toString(),
      workspaceId: json['workspaceId']?.toString(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      visibility: json['visibility']?.toString(),
      driveAccessMode: json['driveAccessMode']?.toString(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'projectId': projectId,
      'workspaceId': workspaceId,
      'name': name,
      'description': description,
      'visibility': visibility,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
    };
  }
}

class ImportAgentProjectRequest {
  final String workspaceId;
  final String? projectId;
  final String name;
  final String? description;
  final String sourceKind;
  final String sourceRef;
  final String driveSpaceId;
  final String driveRootEntryId;
  final String? driveLogicalPath;

  ImportAgentProjectRequest({
    required this.workspaceId,
    this.projectId,
    required this.name,
    this.description,
    required this.sourceKind,
    required this.sourceRef,
    required this.driveSpaceId,
    required this.driveRootEntryId,
    this.driveLogicalPath
  });

  factory ImportAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return ImportAgentProjectRequest(
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.workspaceId is required');
        }
        return value;
      })(),
      projectId: json['projectId']?.toString(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      sourceKind: (() {
        final value = json['sourceKind']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.sourceKind is required');
        }
        return value;
      })(),
      sourceRef: (() {
        final value = json['sourceRef']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.sourceRef is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.driveSpaceId is required');
        }
        return value;
      })(),
      driveRootEntryId: (() {
        final value = json['driveRootEntryId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.driveRootEntryId is required');
        }
        return value;
      })(),
      driveLogicalPath: json['driveLogicalPath']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'workspaceId': workspaceId,
      'projectId': projectId,
      'name': name,
      'description': description,
      'sourceKind': sourceKind,
      'sourceRef': sourceRef,
      'driveSpaceId': driveSpaceId,
      'driveRootEntryId': driveRootEntryId,
      'driveLogicalPath': driveLogicalPath,
    };
  }
}

class UpdateAgentProjectRequest {
  final String? expectedVersion;
  final String? name;
  final String? description;
  final String? visibility;
  final String? driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;

  UpdateAgentProjectRequest({
    this.expectedVersion,
    this.name,
    this.description,
    this.visibility,
    this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId
  });

  factory UpdateAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentProjectRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      name: json['name']?.toString(),
      description: json['description']?.toString(),
      visibility: json['visibility']?.toString(),
      driveAccessMode: json['driveAccessMode']?.toString(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'name': name,
      'description': description,
      'visibility': visibility,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
    };
  }
}

class AgentProjectMutationRequest {
  final String? expectedVersion;

  AgentProjectMutationRequest({
    this.expectedVersion
  });

  factory AgentProjectMutationRequest.fromJson(Map<String, dynamic> json) {
    return AgentProjectMutationRequest(
      expectedVersion: json['expectedVersion']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
    };
  }
}

class AgentProjectCompositionSlotRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String projectId;
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int priority;
  final bool enabled;
  final String policyJson;
  final String createdBy;
  final String updatedBy;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentProjectCompositionSlotRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.projectId,
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.priority,
    required this.enabled,
    required this.policyJson,
    required this.createdBy,
    required this.updatedBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentProjectCompositionSlotRecord.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.organizationId is required');
        }
        return value;
      })(),
      projectId: (() {
        final value = json['projectId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.projectId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotRecord.priority is required');
        }
        return value;
      })(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('AgentProjectCompositionSlotRecord.enabled is required');
        }
        return value;
      })(),
      policyJson: (() {
        final value = json['policyJson']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.policyJson is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.createdBy is required');
        }
        return value;
      })(),
      updatedBy: (() {
        final value = json['updatedBy']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.updatedBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'projectId': projectId,
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'createdBy': createdBy,
      'updatedBy': updatedBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentProjectCompositionSlotResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectCompositionSlotResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectCompositionSlotResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectCompositionSlotResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProjectCompositionSlotListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectCompositionSlotListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectCompositionSlotListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectCompositionSlotListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentProjectCompositionSlotRequest {
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;

  CreateAgentProjectCompositionSlotRequest({
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson
  });

  factory CreateAgentProjectCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProjectCompositionSlotRequest(
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
    };
  }
}

class UpdateAgentProjectCompositionSlotRequest {
  final String expectedVersion;
  final String? slotKind;
  final String? targetModule;
  final String? targetRef;
  final String? targetVersionRef;
  final bool? clearTargetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;

  UpdateAgentProjectCompositionSlotRequest({
    required this.expectedVersion,
    this.slotKind,
    this.targetModule,
    this.targetRef,
    this.targetVersionRef,
    this.clearTargetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson
  });

  factory UpdateAgentProjectCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentProjectCompositionSlotRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentProjectCompositionSlotRequest.expectedVersion is required');
        }
        return value;
      })(),
      slotKind: json['slotKind']?.toString(),
      targetModule: json['targetModule']?.toString(),
      targetRef: json['targetRef']?.toString(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      clearTargetVersionRef: json['clearTargetVersionRef'] is bool ? json['clearTargetVersionRef'] : null,
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'clearTargetVersionRef': clearTargetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
    };
  }
}

class AgentResourceUserStateRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String userId;
  final String resourceType;
  final String resourceId;
  final String? pinnedAt;
  final String? hiddenAt;
  final String? lastOpenedAt;
  final String? lastReadItemSequence;
  final String? customTitle;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentResourceUserStateRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.userId,
    required this.resourceType,
    required this.resourceId,
    this.pinnedAt,
    this.hiddenAt,
    this.lastOpenedAt,
    this.lastReadItemSequence,
    this.customTitle,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentResourceUserStateRecord.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.organizationId is required');
        }
        return value;
      })(),
      userId: (() {
        final value = json['userId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.userId is required');
        }
        return value;
      })(),
      resourceType: (() {
        final value = json['resourceType']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.resourceType is required');
        }
        return value;
      })(),
      resourceId: (() {
        final value = json['resourceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.resourceId is required');
        }
        return value;
      })(),
      pinnedAt: json['pinnedAt']?.toString(),
      hiddenAt: json['hiddenAt']?.toString(),
      lastOpenedAt: json['lastOpenedAt']?.toString(),
      lastReadItemSequence: json['lastReadItemSequence']?.toString(),
      customTitle: json['customTitle']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'userId': userId,
      'resourceType': resourceType,
      'resourceId': resourceId,
      'pinnedAt': pinnedAt,
      'hiddenAt': hiddenAt,
      'lastOpenedAt': lastOpenedAt,
      'lastReadItemSequence': lastReadItemSequence,
      'customTitle': customTitle,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentResourceUserStateResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResourceUserStateResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResourceUserStateResponse.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResourceUserStateResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResourceUserStateResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentResourceUserStateListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResourceUserStateListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResourceUserStateListResponse.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResourceUserStateListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResourceUserStateListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class UpdateAgentSessionUserStateRequest {
  final String? expectedVersion;
  final bool? pinned;
  final bool? hidden;
  final bool? markOpened;
  final String? lastReadItemSequence;
  final String? customTitle;
  final bool? clearCustomTitle;

  UpdateAgentSessionUserStateRequest({
    this.expectedVersion,
    this.pinned,
    this.hidden,
    this.markOpened,
    this.lastReadItemSequence,
    this.customTitle,
    this.clearCustomTitle
  });

  factory UpdateAgentSessionUserStateRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentSessionUserStateRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      pinned: json['pinned'] is bool ? json['pinned'] : null,
      hidden: json['hidden'] is bool ? json['hidden'] : null,
      markOpened: json['markOpened'] is bool ? json['markOpened'] : null,
      lastReadItemSequence: json['lastReadItemSequence']?.toString(),
      customTitle: json['customTitle']?.toString(),
      clearCustomTitle: json['clearCustomTitle'] is bool ? json['clearCustomTitle'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'pinned': pinned,
      'hidden': hidden,
      'markOpened': markOpened,
      'lastReadItemSequence': lastReadItemSequence,
      'customTitle': customTitle,
      'clearCustomTitle': clearCustomTitle,
    };
  }
}

class AgentItemFeedbackRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String itemId;
  final String userId;
  final String rating;
  final String? reasonCode;
  final String? comment;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentItemFeedbackRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.itemId,
    required this.userId,
    required this.rating,
    this.reasonCode,
    this.comment,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentItemFeedbackRecord.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.organizationId is required');
        }
        return value;
      })(),
      itemId: (() {
        final value = json['itemId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.itemId is required');
        }
        return value;
      })(),
      userId: (() {
        final value = json['userId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.userId is required');
        }
        return value;
      })(),
      rating: (() {
        final value = json['rating']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.rating is required');
        }
        return value;
      })(),
      reasonCode: json['reasonCode']?.toString(),
      comment: json['comment']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'itemId': itemId,
      'userId': userId,
      'rating': rating,
      'reasonCode': reasonCode,
      'comment': comment,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentItemFeedbackResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentItemFeedbackResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentItemFeedbackResponse.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentItemFeedbackResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentItemFeedbackResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentItemFeedbackListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentItemFeedbackListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentItemFeedbackListResponse.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentItemFeedbackListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentItemFeedbackListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class UpdateAgentItemFeedbackRequest {
  final String? expectedVersion;
  final String? rating;
  final bool? clearFeedback;
  final String? reasonCode;
  final String? comment;

  UpdateAgentItemFeedbackRequest({
    this.expectedVersion,
    this.rating,
    this.clearFeedback,
    this.reasonCode,
    this.comment
  });

  factory UpdateAgentItemFeedbackRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentItemFeedbackRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      rating: json['rating']?.toString(),
      clearFeedback: json['clearFeedback'] is bool ? json['clearFeedback'] : null,
      reasonCode: json['reasonCode']?.toString(),
      comment: json['comment']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'rating': rating,
      'clearFeedback': clearFeedback,
      'reasonCode': reasonCode,
      'comment': comment,
    };
  }
}

class AgentTaskRecord {
  final String taskId;
  final String agentId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String sessionId;
  final String? title;
  final String prompt;
  final String scheduleKind;
  final String? cronExpression;
  final String timezone;
  final String? scheduledAt;
  final String? startsAt;
  final String? endsAt;
  final String? nextFireAt;
  final String misfirePolicy;
  final String overlapPolicy;
  final int maxConcurrentRuns;
  final int maxCatchUpRuns;
  final int maxAttempts;
  final int retryInitialDelaySeconds;
  final int retryMaxDelaySeconds;
  final int timeoutSeconds;
  final int priority;
  final String status;
  final String generation;
  final String? externalRef;
  final String metadataJson;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? completedAt;
  final String? pausedAt;
  final String? cancelledAt;

  AgentTaskRecord({
    required this.taskId,
    required this.agentId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.sessionId,
    required this.title,
    required this.prompt,
    required this.scheduleKind,
    required this.cronExpression,
    required this.timezone,
    required this.scheduledAt,
    required this.startsAt,
    required this.endsAt,
    required this.nextFireAt,
    required this.misfirePolicy,
    required this.overlapPolicy,
    required this.maxConcurrentRuns,
    required this.maxCatchUpRuns,
    required this.maxAttempts,
    required this.retryInitialDelaySeconds,
    required this.retryMaxDelaySeconds,
    required this.timeoutSeconds,
    required this.priority,
    required this.status,
    required this.generation,
    required this.externalRef,
    required this.metadataJson,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    required this.completedAt,
    required this.pausedAt,
    required this.cancelledAt
  });

  factory AgentTaskRecord.fromJson(Map<String, dynamic> json) {
    return AgentTaskRecord(
      taskId: (() {
        final value = json['taskId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.taskId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.agentId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.ownerUserId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.sessionId is required');
        }
        return value;
      })(),
      title: (() {
        if (!json.containsKey('title')) {
          throw FormatException('AgentTaskRecord.title is required');
        }
        final _sdkworkRequiredValue = json['title'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.title is required');
        }
        return value;
      })();
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.prompt is required');
        }
        return value;
      })(),
      scheduleKind: (() {
        final value = json['scheduleKind']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.scheduleKind is required');
        }
        return value;
      })(),
      cronExpression: (() {
        if (!json.containsKey('cronExpression')) {
          throw FormatException('AgentTaskRecord.cronExpression is required');
        }
        final _sdkworkRequiredValue = json['cronExpression'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.cronExpression is required');
        }
        return value;
      })();
      })(),
      timezone: (() {
        final value = json['timezone']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.timezone is required');
        }
        return value;
      })(),
      scheduledAt: (() {
        if (!json.containsKey('scheduledAt')) {
          throw FormatException('AgentTaskRecord.scheduledAt is required');
        }
        final _sdkworkRequiredValue = json['scheduledAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.scheduledAt is required');
        }
        return value;
      })();
      })(),
      startsAt: (() {
        if (!json.containsKey('startsAt')) {
          throw FormatException('AgentTaskRecord.startsAt is required');
        }
        final _sdkworkRequiredValue = json['startsAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.startsAt is required');
        }
        return value;
      })();
      })(),
      endsAt: (() {
        if (!json.containsKey('endsAt')) {
          throw FormatException('AgentTaskRecord.endsAt is required');
        }
        final _sdkworkRequiredValue = json['endsAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.endsAt is required');
        }
        return value;
      })();
      })(),
      nextFireAt: (() {
        if (!json.containsKey('nextFireAt')) {
          throw FormatException('AgentTaskRecord.nextFireAt is required');
        }
        final _sdkworkRequiredValue = json['nextFireAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.nextFireAt is required');
        }
        return value;
      })();
      })(),
      misfirePolicy: (() {
        final value = json['misfirePolicy']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.misfirePolicy is required');
        }
        return value;
      })(),
      overlapPolicy: (() {
        final value = json['overlapPolicy']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.overlapPolicy is required');
        }
        return value;
      })(),
      maxConcurrentRuns: (() {
        final value = json['maxConcurrentRuns'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.maxConcurrentRuns is required');
        }
        return value;
      })(),
      maxCatchUpRuns: (() {
        final value = json['maxCatchUpRuns'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.maxCatchUpRuns is required');
        }
        return value;
      })(),
      maxAttempts: (() {
        final value = json['maxAttempts'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.maxAttempts is required');
        }
        return value;
      })(),
      retryInitialDelaySeconds: (() {
        final value = json['retryInitialDelaySeconds'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.retryInitialDelaySeconds is required');
        }
        return value;
      })(),
      retryMaxDelaySeconds: (() {
        final value = json['retryMaxDelaySeconds'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.retryMaxDelaySeconds is required');
        }
        return value;
      })(),
      timeoutSeconds: (() {
        final value = json['timeoutSeconds'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.timeoutSeconds is required');
        }
        return value;
      })(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentTaskRecord.priority is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.status is required');
        }
        return value;
      })(),
      generation: (() {
        final value = json['generation']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.generation is required');
        }
        return value;
      })(),
      externalRef: (() {
        if (!json.containsKey('externalRef')) {
          throw FormatException('AgentTaskRecord.externalRef is required');
        }
        final _sdkworkRequiredValue = json['externalRef'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.externalRef is required');
        }
        return value;
      })();
      })(),
      metadataJson: (() {
        final value = json['metadataJson']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.metadataJson is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.updatedAt is required');
        }
        return value;
      })(),
      completedAt: (() {
        if (!json.containsKey('completedAt')) {
          throw FormatException('AgentTaskRecord.completedAt is required');
        }
        final _sdkworkRequiredValue = json['completedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.completedAt is required');
        }
        return value;
      })();
      })(),
      pausedAt: (() {
        if (!json.containsKey('pausedAt')) {
          throw FormatException('AgentTaskRecord.pausedAt is required');
        }
        final _sdkworkRequiredValue = json['pausedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.pausedAt is required');
        }
        return value;
      })();
      })(),
      cancelledAt: (() {
        if (!json.containsKey('cancelledAt')) {
          throw FormatException('AgentTaskRecord.cancelledAt is required');
        }
        final _sdkworkRequiredValue = json['cancelledAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.cancelledAt is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'taskId': taskId,
      'agentId': agentId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'sessionId': sessionId,
      'title': title,
      'prompt': prompt,
      'scheduleKind': scheduleKind,
      'cronExpression': cronExpression,
      'timezone': timezone,
      'scheduledAt': scheduledAt,
      'startsAt': startsAt,
      'endsAt': endsAt,
      'nextFireAt': nextFireAt,
      'misfirePolicy': misfirePolicy,
      'overlapPolicy': overlapPolicy,
      'maxConcurrentRuns': maxConcurrentRuns,
      'maxCatchUpRuns': maxCatchUpRuns,
      'maxAttempts': maxAttempts,
      'retryInitialDelaySeconds': retryInitialDelaySeconds,
      'retryMaxDelaySeconds': retryMaxDelaySeconds,
      'timeoutSeconds': timeoutSeconds,
      'priority': priority,
      'status': status,
      'generation': generation,
      'externalRef': externalRef,
      'metadataJson': metadataJson,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'completedAt': completedAt,
      'pausedAt': pausedAt,
      'cancelledAt': cancelledAt,
    };
  }
}

class AgentTaskResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTaskListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentTaskRequest {
  final String? taskId;
  final String sessionId;
  final String title;
  final String prompt;
  final String scheduleKind;
  final String? cronExpression;
  final String timezone;
  final String? scheduledAt;
  final String? startsAt;
  final String? endsAt;
  final String? misfirePolicy;
  final String? overlapPolicy;
  final int? maxConcurrentRuns;
  final int? maxCatchUpRuns;
  final int? maxAttempts;
  final int? retryInitialDelaySeconds;
  final int? retryMaxDelaySeconds;
  final int? timeoutSeconds;
  final int? priority;
  final String? externalRef;
  final String? metadataJson;
  final String requestedAt;

  CreateAgentTaskRequest({
    this.taskId,
    required this.sessionId,
    required this.title,
    required this.prompt,
    required this.scheduleKind,
    this.cronExpression,
    required this.timezone,
    this.scheduledAt,
    this.startsAt,
    this.endsAt,
    this.misfirePolicy,
    this.overlapPolicy,
    this.maxConcurrentRuns,
    this.maxCatchUpRuns,
    this.maxAttempts,
    this.retryInitialDelaySeconds,
    this.retryMaxDelaySeconds,
    this.timeoutSeconds,
    this.priority,
    this.externalRef,
    this.metadataJson,
    required this.requestedAt
  });

  factory CreateAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentTaskRequest(
      taskId: json['taskId']?.toString(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.sessionId is required');
        }
        return value;
      })(),
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.title is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.prompt is required');
        }
        return value;
      })(),
      scheduleKind: (() {
        final value = json['scheduleKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.scheduleKind is required');
        }
        return value;
      })(),
      cronExpression: json['cronExpression']?.toString(),
      timezone: (() {
        final value = json['timezone']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.timezone is required');
        }
        return value;
      })(),
      scheduledAt: json['scheduledAt']?.toString(),
      startsAt: json['startsAt']?.toString(),
      endsAt: json['endsAt']?.toString(),
      misfirePolicy: json['misfirePolicy']?.toString(),
      overlapPolicy: json['overlapPolicy']?.toString(),
      maxConcurrentRuns: json['maxConcurrentRuns'] is int ? json['maxConcurrentRuns'] : null,
      maxCatchUpRuns: json['maxCatchUpRuns'] is int ? json['maxCatchUpRuns'] : null,
      maxAttempts: json['maxAttempts'] is int ? json['maxAttempts'] : null,
      retryInitialDelaySeconds: json['retryInitialDelaySeconds'] is int ? json['retryInitialDelaySeconds'] : null,
      retryMaxDelaySeconds: json['retryMaxDelaySeconds'] is int ? json['retryMaxDelaySeconds'] : null,
      timeoutSeconds: json['timeoutSeconds'] is int ? json['timeoutSeconds'] : null,
      priority: json['priority'] is int ? json['priority'] : null,
      externalRef: json['externalRef']?.toString(),
      metadataJson: json['metadataJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'taskId': taskId,
      'sessionId': sessionId,
      'title': title,
      'prompt': prompt,
      'scheduleKind': scheduleKind,
      'cronExpression': cronExpression,
      'timezone': timezone,
      'scheduledAt': scheduledAt,
      'startsAt': startsAt,
      'endsAt': endsAt,
      'misfirePolicy': misfirePolicy,
      'overlapPolicy': overlapPolicy,
      'maxConcurrentRuns': maxConcurrentRuns,
      'maxCatchUpRuns': maxCatchUpRuns,
      'maxAttempts': maxAttempts,
      'retryInitialDelaySeconds': retryInitialDelaySeconds,
      'retryMaxDelaySeconds': retryMaxDelaySeconds,
      'timeoutSeconds': timeoutSeconds,
      'priority': priority,
      'externalRef': externalRef,
      'metadataJson': metadataJson,
      'requestedAt': requestedAt,
    };
  }
}

class ReplaceAgentTaskRequest {
  final String title;
  final String prompt;
  final String scheduleKind;
  final String? cronExpression;
  final String timezone;
  final String? scheduledAt;
  final String? startsAt;
  final String? endsAt;
  final String? misfirePolicy;
  final String? overlapPolicy;
  final int? maxConcurrentRuns;
  final int? maxCatchUpRuns;
  final int? maxAttempts;
  final int? retryInitialDelaySeconds;
  final int? retryMaxDelaySeconds;
  final int? timeoutSeconds;
  final int? priority;
  final String? externalRef;
  final String? metadataJson;
  final String expectedVersion;
  final String requestedAt;

  ReplaceAgentTaskRequest({
    required this.title,
    required this.prompt,
    required this.scheduleKind,
    this.cronExpression,
    required this.timezone,
    this.scheduledAt,
    this.startsAt,
    this.endsAt,
    this.misfirePolicy,
    this.overlapPolicy,
    this.maxConcurrentRuns,
    this.maxCatchUpRuns,
    this.maxAttempts,
    this.retryInitialDelaySeconds,
    this.retryMaxDelaySeconds,
    this.timeoutSeconds,
    this.priority,
    this.externalRef,
    this.metadataJson,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ReplaceAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return ReplaceAgentTaskRequest(
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.title is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.prompt is required');
        }
        return value;
      })(),
      scheduleKind: (() {
        final value = json['scheduleKind']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.scheduleKind is required');
        }
        return value;
      })(),
      cronExpression: json['cronExpression']?.toString(),
      timezone: (() {
        final value = json['timezone']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.timezone is required');
        }
        return value;
      })(),
      scheduledAt: json['scheduledAt']?.toString(),
      startsAt: json['startsAt']?.toString(),
      endsAt: json['endsAt']?.toString(),
      misfirePolicy: json['misfirePolicy']?.toString(),
      overlapPolicy: json['overlapPolicy']?.toString(),
      maxConcurrentRuns: json['maxConcurrentRuns'] is int ? json['maxConcurrentRuns'] : null,
      maxCatchUpRuns: json['maxCatchUpRuns'] is int ? json['maxCatchUpRuns'] : null,
      maxAttempts: json['maxAttempts'] is int ? json['maxAttempts'] : null,
      retryInitialDelaySeconds: json['retryInitialDelaySeconds'] is int ? json['retryInitialDelaySeconds'] : null,
      retryMaxDelaySeconds: json['retryMaxDelaySeconds'] is int ? json['retryMaxDelaySeconds'] : null,
      timeoutSeconds: json['timeoutSeconds'] is int ? json['timeoutSeconds'] : null,
      priority: json['priority'] is int ? json['priority'] : null,
      externalRef: json['externalRef']?.toString(),
      metadataJson: json['metadataJson']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ReplaceAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'title': title,
      'prompt': prompt,
      'scheduleKind': scheduleKind,
      'cronExpression': cronExpression,
      'timezone': timezone,
      'scheduledAt': scheduledAt,
      'startsAt': startsAt,
      'endsAt': endsAt,
      'misfirePolicy': misfirePolicy,
      'overlapPolicy': overlapPolicy,
      'maxConcurrentRuns': maxConcurrentRuns,
      'maxCatchUpRuns': maxCatchUpRuns,
      'maxAttempts': maxAttempts,
      'retryInitialDelaySeconds': retryInitialDelaySeconds,
      'retryMaxDelaySeconds': retryMaxDelaySeconds,
      'timeoutSeconds': timeoutSeconds,
      'priority': priority,
      'externalRef': externalRef,
      'metadataJson': metadataJson,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentTaskStateChangeRequest {
  final String expectedVersion;
  final String requestedAt;

  AgentTaskStateChangeRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory AgentTaskStateChangeRequest.fromJson(Map<String, dynamic> json) {
    return AgentTaskStateChangeRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskStateChangeRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskStateChangeRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class CancelAgentTaskRequest {
  final String? expectedVersion;
  final String requestedAt;

  CancelAgentTaskRequest({
    this.expectedVersion,
    required this.requestedAt
  });

  factory CancelAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return CancelAgentTaskRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AppUpdateAgentSessionRequest {
  final String? expectedVersion;
  final String? title;
  final String? projectId;
  final bool? clearProject;

  AppUpdateAgentSessionRequest({
    this.expectedVersion,
    this.title,
    this.projectId,
    this.clearProject
  });

  factory AppUpdateAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return AppUpdateAgentSessionRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      title: json['title']?.toString(),
      projectId: json['projectId']?.toString(),
      clearProject: json['clearProject'] is bool ? json['clearProject'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'title': title,
      'projectId': projectId,
      'clearProject': clearProject,
    };
  }
}

class ApplyAgentModelConfigurationRequest {
  final String configurationId;
  final String engineId;
  final String? agentId;
  final String vendorCode;
  final String baseUrl;
  final String? apiKey;
  final String defaultModelId;
  final List<String> supportedModelIds;
  final List<String>? supportedProviderIds;
  final int? inputContextTokens;
  final int? outputContextTokens;
  final int? toolCallRounds;
  final bool? supportsMultimodal;

  ApplyAgentModelConfigurationRequest({
    required this.configurationId,
    required this.engineId,
    this.agentId,
    required this.vendorCode,
    required this.baseUrl,
    this.apiKey,
    required this.defaultModelId,
    required this.supportedModelIds,
    this.supportedProviderIds,
    this.inputContextTokens,
    this.outputContextTokens,
    this.toolCallRounds,
    this.supportsMultimodal
  });

  factory ApplyAgentModelConfigurationRequest.fromJson(Map<String, dynamic> json) {
    return ApplyAgentModelConfigurationRequest(
      configurationId: (() {
        final value = json['configurationId']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.configurationId is required');
        }
        return value;
      })(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.engineId is required');
        }
        return value;
      })(),
      agentId: json['agentId']?.toString(),
      vendorCode: (() {
        final value = json['vendorCode']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.vendorCode is required');
        }
        return value;
      })(),
      baseUrl: (() {
        final value = json['baseUrl']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.baseUrl is required');
        }
        return value;
      })(),
      apiKey: json['apiKey']?.toString(),
      defaultModelId: (() {
        final value = json['defaultModelId']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.defaultModelId is required');
        }
        return value;
      })(),
      supportedModelIds: (() {
        final list = _sdkworkAsList(json['supportedModelIds']);
        if (list == null) {
          throw FormatException('ApplyAgentModelConfigurationRequest.supportedModelIds is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      supportedProviderIds: (() {
        final list = _sdkworkAsList(json['supportedProviderIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      inputContextTokens: json['inputContextTokens'] is int ? json['inputContextTokens'] : null,
      outputContextTokens: json['outputContextTokens'] is int ? json['outputContextTokens'] : null,
      toolCallRounds: json['toolCallRounds'] is int ? json['toolCallRounds'] : null,
      supportsMultimodal: json['supportsMultimodal'] is bool ? json['supportsMultimodal'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configurationId': configurationId,
      'engineId': engineId,
      'agentId': agentId,
      'vendorCode': vendorCode,
      'baseUrl': baseUrl,
      'apiKey': apiKey,
      'defaultModelId': defaultModelId,
      'supportedModelIds': supportedModelIds.map((item) => item).toList(),
      'supportedProviderIds': supportedProviderIds?.map((item) => item).toList(),
      'inputContextTokens': inputContextTokens,
      'outputContextTokens': outputContextTokens,
      'toolCallRounds': toolCallRounds,
      'supportsMultimodal': supportsMultimodal,
    };
  }
}

class AppliedAgentModelConfigurationRecord {
  final String configurationId;
  final String profileId;
  final String engineId;
  final String agentId;
  final String providerScope;
  final String vendorCode;
  final String baseUrl;
  final String defaultModelId;
  final List<String> supportedModelIds;
  final List<String> supportedProviderIds;
  final int? inputContextTokens;
  final int? outputContextTokens;
  final int? toolCallRounds;
  final bool supportsMultimodal;
  final bool apiKeyConfigured;

  AppliedAgentModelConfigurationRecord({
    required this.configurationId,
    required this.profileId,
    required this.engineId,
    required this.agentId,
    required this.providerScope,
    required this.vendorCode,
    required this.baseUrl,
    required this.defaultModelId,
    required this.supportedModelIds,
    required this.supportedProviderIds,
    this.inputContextTokens,
    this.outputContextTokens,
    this.toolCallRounds,
    required this.supportsMultimodal,
    required this.apiKeyConfigured
  });

  factory AppliedAgentModelConfigurationRecord.fromJson(Map<String, dynamic> json) {
    return AppliedAgentModelConfigurationRecord(
      configurationId: (() {
        final value = json['configurationId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.configurationId is required');
        }
        return value;
      })(),
      profileId: (() {
        final value = json['profileId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.profileId is required');
        }
        return value;
      })(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.engineId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.agentId is required');
        }
        return value;
      })(),
      providerScope: (() {
        final value = json['providerScope']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.providerScope is required');
        }
        return value;
      })(),
      vendorCode: (() {
        final value = json['vendorCode']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.vendorCode is required');
        }
        return value;
      })(),
      baseUrl: (() {
        final value = json['baseUrl']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.baseUrl is required');
        }
        return value;
      })(),
      defaultModelId: (() {
        final value = json['defaultModelId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.defaultModelId is required');
        }
        return value;
      })(),
      supportedModelIds: (() {
        final list = _sdkworkAsList(json['supportedModelIds']);
        if (list == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.supportedModelIds is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      supportedProviderIds: (() {
        final list = _sdkworkAsList(json['supportedProviderIds']);
        if (list == null) {
          throw FormatException('AppliedAgentModelConfigurationRecord.supportedProviderIds is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      inputContextTokens: json['inputContextTokens'] is int ? json['inputContextTokens'] : null,
      outputContextTokens: json['outputContextTokens'] is int ? json['outputContextTokens'] : null,
      toolCallRounds: json['toolCallRounds'] is int ? json['toolCallRounds'] : null,
      supportsMultimodal: (() {
        final value = json['supportsMultimodal'];
        if (value is! bool) {
          throw FormatException('AppliedAgentModelConfigurationRecord.supportsMultimodal is required');
        }
        return value;
      })(),
      apiKeyConfigured: (() {
        final value = json['apiKeyConfigured'];
        if (value is! bool) {
          throw FormatException('AppliedAgentModelConfigurationRecord.apiKeyConfigured is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configurationId': configurationId,
      'profileId': profileId,
      'engineId': engineId,
      'agentId': agentId,
      'providerScope': providerScope,
      'vendorCode': vendorCode,
      'baseUrl': baseUrl,
      'defaultModelId': defaultModelId,
      'supportedModelIds': supportedModelIds.map((item) => item).toList(),
      'supportedProviderIds': supportedProviderIds.map((item) => item).toList(),
      'inputContextTokens': inputContextTokens,
      'outputContextTokens': outputContextTokens,
      'toolCallRounds': toolCallRounds,
      'supportsMultimodal': supportsMultimodal,
      'apiKeyConfigured': apiKeyConfigured,
    };
  }
}

class AppliedAgentModelConfigurationResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AppliedAgentModelConfigurationResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AppliedAgentModelConfigurationResponse.fromJson(Map<String, dynamic> json) {
    return AppliedAgentModelConfigurationResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AppliedAgentModelConfigurationResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AppliedAgentModelConfigurationResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelConfigurationResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ModelConfigurationSummaryRecord {
  final String profileId;
  final String engineId;
  final String agentId;
  final String providerScope;
  final String configurationVersion;
  final String status;
  final String baseUrl;
  final String defaultModelId;
  final List<String> supportedModelIds;
  final bool apiKeyConfigured;

  ModelConfigurationSummaryRecord({
    required this.profileId,
    required this.engineId,
    required this.agentId,
    required this.providerScope,
    required this.configurationVersion,
    required this.status,
    required this.baseUrl,
    required this.defaultModelId,
    required this.supportedModelIds,
    required this.apiKeyConfigured
  });

  factory ModelConfigurationSummaryRecord.fromJson(Map<String, dynamic> json) {
    return ModelConfigurationSummaryRecord(
      profileId: (() {
        final value = json['profileId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.profileId is required');
        }
        return value;
      })(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.engineId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.agentId is required');
        }
        return value;
      })(),
      providerScope: (() {
        final value = json['providerScope']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.providerScope is required');
        }
        return value;
      })(),
      configurationVersion: (() {
        final value = json['configurationVersion']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.configurationVersion is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.status is required');
        }
        return value;
      })(),
      baseUrl: (() {
        final value = json['baseUrl']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.baseUrl is required');
        }
        return value;
      })(),
      defaultModelId: (() {
        final value = json['defaultModelId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryRecord.defaultModelId is required');
        }
        return value;
      })(),
      supportedModelIds: (() {
        final list = _sdkworkAsList(json['supportedModelIds']);
        if (list == null) {
          throw FormatException('ModelConfigurationSummaryRecord.supportedModelIds is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      apiKeyConfigured: (() {
        final value = json['apiKeyConfigured'];
        if (value is! bool) {
          throw FormatException('ModelConfigurationSummaryRecord.apiKeyConfigured is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'profileId': profileId,
      'engineId': engineId,
      'agentId': agentId,
      'providerScope': providerScope,
      'configurationVersion': configurationVersion,
      'status': status,
      'baseUrl': baseUrl,
      'defaultModelId': defaultModelId,
      'supportedModelIds': supportedModelIds.map((item) => item).toList(),
      'apiKeyConfigured': apiKeyConfigured,
    };
  }
}

class ModelConfigurationSummaryResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ModelConfigurationSummaryResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ModelConfigurationSummaryResponse.fromJson(Map<String, dynamic> json) {
    return ModelConfigurationSummaryResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ModelConfigurationSummaryResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ModelConfigurationSummaryResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ModelConfigurationSummaryListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ModelConfigurationSummaryListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ModelConfigurationSummaryListResponse.fromJson(Map<String, dynamic> json) {
    return ModelConfigurationSummaryListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ModelConfigurationSummaryListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ModelConfigurationSummaryListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationSummaryListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ModelConfigurationStatusRecord {
  final String profileId;
  final String engineId;
  final String agentId;
  final String providerScope;
  final String status;
  final String materialization;
  final String derivedState;
  final String? expectedBaseUrl;
  final String? expectedDefaultModel;
  final String? effectiveBaseUrl;
  final String? effectiveDefaultModel;
  final bool credentialConfigured;
  final List<String> issues;

  ModelConfigurationStatusRecord({
    required this.profileId,
    required this.engineId,
    required this.agentId,
    required this.providerScope,
    required this.status,
    required this.materialization,
    required this.derivedState,
    this.expectedBaseUrl,
    this.expectedDefaultModel,
    this.effectiveBaseUrl,
    this.effectiveDefaultModel,
    required this.credentialConfigured,
    required this.issues
  });

  factory ModelConfigurationStatusRecord.fromJson(Map<String, dynamic> json) {
    return ModelConfigurationStatusRecord(
      profileId: (() {
        final value = json['profileId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.profileId is required');
        }
        return value;
      })(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.engineId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.agentId is required');
        }
        return value;
      })(),
      providerScope: (() {
        final value = json['providerScope']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.providerScope is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.status is required');
        }
        return value;
      })(),
      materialization: (() {
        final value = json['materialization']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.materialization is required');
        }
        return value;
      })(),
      derivedState: (() {
        final value = json['derivedState']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusRecord.derivedState is required');
        }
        return value;
      })(),
      expectedBaseUrl: json['expectedBaseUrl']?.toString(),
      expectedDefaultModel: json['expectedDefaultModel']?.toString(),
      effectiveBaseUrl: json['effectiveBaseUrl']?.toString(),
      effectiveDefaultModel: json['effectiveDefaultModel']?.toString(),
      credentialConfigured: (() {
        final value = json['credentialConfigured'];
        if (value is! bool) {
          throw FormatException('ModelConfigurationStatusRecord.credentialConfigured is required');
        }
        return value;
      })(),
      issues: (() {
        final list = _sdkworkAsList(json['issues']);
        if (list == null) {
          throw FormatException('ModelConfigurationStatusRecord.issues is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'profileId': profileId,
      'engineId': engineId,
      'agentId': agentId,
      'providerScope': providerScope,
      'status': status,
      'materialization': materialization,
      'derivedState': derivedState,
      'expectedBaseUrl': expectedBaseUrl,
      'expectedDefaultModel': expectedDefaultModel,
      'effectiveBaseUrl': effectiveBaseUrl,
      'effectiveDefaultModel': effectiveDefaultModel,
      'credentialConfigured': credentialConfigured,
      'issues': issues.map((item) => item).toList(),
    };
  }
}

class ModelConfigurationStatusResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ModelConfigurationStatusResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ModelConfigurationStatusResponse.fromJson(Map<String, dynamic> json) {
    return ModelConfigurationStatusResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ModelConfigurationStatusResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ModelConfigurationStatusResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ModelConfigurationStatusResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class MigrateModelConfigurationRequest {
  final String engineId;
  final String profileId;
  final String fromConfigurationVersion;
  final String toConfigurationVersion;

  MigrateModelConfigurationRequest({
    required this.engineId,
    required this.profileId,
    required this.fromConfigurationVersion,
    required this.toConfigurationVersion
  });

  factory MigrateModelConfigurationRequest.fromJson(Map<String, dynamic> json) {
    return MigrateModelConfigurationRequest(
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('MigrateModelConfigurationRequest.engineId is required');
        }
        return value;
      })(),
      profileId: (() {
        final value = json['profileId']?.toString();
        if (value == null) {
          throw FormatException('MigrateModelConfigurationRequest.profileId is required');
        }
        return value;
      })(),
      fromConfigurationVersion: (() {
        final value = json['fromConfigurationVersion']?.toString();
        if (value == null) {
          throw FormatException('MigrateModelConfigurationRequest.fromConfigurationVersion is required');
        }
        return value;
      })(),
      toConfigurationVersion: (() {
        final value = json['toConfigurationVersion']?.toString();
        if (value == null) {
          throw FormatException('MigrateModelConfigurationRequest.toConfigurationVersion is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineId': engineId,
      'profileId': profileId,
      'fromConfigurationVersion': fromConfigurationVersion,
      'toConfigurationVersion': toConfigurationVersion,
    };
  }
}

class MigratedModelConfigurationResponse {
  final int code;
  final dynamic data;
  final String traceId;

  MigratedModelConfigurationResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory MigratedModelConfigurationResponse.fromJson(Map<String, dynamic> json) {
    return MigratedModelConfigurationResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('MigratedModelConfigurationResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('MigratedModelConfigurationResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('MigratedModelConfigurationResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ApplyAgentModelSelectionRequest {
  final String? configurationId;
  final String engineId;
  final String? agentId;
  final String modelId;

  ApplyAgentModelSelectionRequest({
    this.configurationId,
    required this.engineId,
    this.agentId,
    required this.modelId
  });

  factory ApplyAgentModelSelectionRequest.fromJson(Map<String, dynamic> json) {
    return ApplyAgentModelSelectionRequest(
      configurationId: json['configurationId']?.toString(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelSelectionRequest.engineId is required');
        }
        return value;
      })(),
      agentId: json['agentId']?.toString(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('ApplyAgentModelSelectionRequest.modelId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configurationId': configurationId,
      'engineId': engineId,
      'agentId': agentId,
      'modelId': modelId,
    };
  }
}

class AppliedAgentModelSelectionRecord {
  final String? configurationId;
  final String profileId;
  final String engineId;
  final String agentId;
  final String providerScope;
  final String modelId;

  AppliedAgentModelSelectionRecord({
    this.configurationId,
    required this.profileId,
    required this.engineId,
    required this.agentId,
    required this.providerScope,
    required this.modelId
  });

  factory AppliedAgentModelSelectionRecord.fromJson(Map<String, dynamic> json) {
    return AppliedAgentModelSelectionRecord(
      configurationId: json['configurationId']?.toString(),
      profileId: (() {
        final value = json['profileId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionRecord.profileId is required');
        }
        return value;
      })(),
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionRecord.engineId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionRecord.agentId is required');
        }
        return value;
      })(),
      providerScope: (() {
        final value = json['providerScope']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionRecord.providerScope is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionRecord.modelId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configurationId': configurationId,
      'profileId': profileId,
      'engineId': engineId,
      'agentId': agentId,
      'providerScope': providerScope,
      'modelId': modelId,
    };
  }
}

class AgentEngineConfigFileResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentEngineConfigFileResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentEngineConfigFileResponse.fromJson(Map<String, dynamic> json) {
    return AgentEngineConfigFileResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentEngineConfigFileResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentEngineConfigFileResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineConfigFileResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentEngineConfigFileView {
  final String engineId;
  final String configFilePath;
  final String format;
  final String content;
  final bool exists;

  AgentEngineConfigFileView({
    required this.engineId,
    required this.configFilePath,
    required this.format,
    required this.content,
    required this.exists
  });

  factory AgentEngineConfigFileView.fromJson(Map<String, dynamic> json) {
    return AgentEngineConfigFileView(
      engineId: (() {
        final value = json['engineId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineConfigFileView.engineId is required');
        }
        return value;
      })(),
      configFilePath: (() {
        final value = json['configFilePath']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineConfigFileView.configFilePath is required');
        }
        return value;
      })(),
      format: (() {
        final value = json['format']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineConfigFileView.format is required');
        }
        return value;
      })(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineConfigFileView.content is required');
        }
        return value;
      })(),
      exists: (() {
        final value = json['exists'];
        if (value is! bool) {
          throw FormatException('AgentEngineConfigFileView.exists is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineId': engineId,
      'configFilePath': configFilePath,
      'format': format,
      'content': content,
      'exists': exists,
    };
  }
}

class AppliedAgentModelSelectionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AppliedAgentModelSelectionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AppliedAgentModelSelectionResponse.fromJson(Map<String, dynamic> json) {
    return AppliedAgentModelSelectionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AppliedAgentModelSelectionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AppliedAgentModelSelectionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AppliedAgentModelSelectionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentEngineCatalogListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentEngineCatalogListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentEngineCatalogListResponse.fromJson(Map<String, dynamic> json) {
    return AgentEngineCatalogListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentEngineCatalogListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentEngineCatalogListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentEngineCatalog {
  final List<AgentEngineCatalogEngine> engines;

  AgentEngineCatalog({
    required this.engines
  });

  factory AgentEngineCatalog.fromJson(Map<String, dynamic> json) {
    return AgentEngineCatalog(
      engines: (() {
        final list = _sdkworkAsList(json['engines']);
        if (list == null) {
          throw FormatException('AgentEngineCatalog.engines is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentEngineCatalogEngine.fromJson(map);
      })())
            .whereType<AgentEngineCatalogEngine>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engines': engines.map((item) => item.toJson()).toList(),
    };
  }
}

class AgentEngineCatalogEngine {
  final String engineKey;
  final String engineKind;
  final String tier;
  final String agentId;
  final String bindingId;
  final List<AgentEngineModelCatalogEntry> models;
  final String defaultAccessModeId;
  final List<AgentEngineAccessModeCatalogEntry> accessModes;
  final bool available;
  final String? unavailableReason;

  AgentEngineCatalogEngine({
    required this.engineKey,
    required this.engineKind,
    required this.tier,
    required this.agentId,
    required this.bindingId,
    required this.models,
    required this.defaultAccessModeId,
    required this.accessModes,
    required this.available,
    this.unavailableReason
  });

  factory AgentEngineCatalogEngine.fromJson(Map<String, dynamic> json) {
    return AgentEngineCatalogEngine(
      engineKey: (() {
        final value = json['engineKey']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.engineKey is required');
        }
        return value;
      })(),
      engineKind: (() {
        final value = json['engineKind']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.engineKind is required');
        }
        return value;
      })(),
      tier: (() {
        final value = json['tier']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.tier is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.agentId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.bindingId is required');
        }
        return value;
      })(),
      models: (() {
        final list = _sdkworkAsList(json['models']);
        if (list == null) {
          throw FormatException('AgentEngineCatalogEngine.models is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentEngineModelCatalogEntry.fromJson(map);
      })())
            .whereType<AgentEngineModelCatalogEntry>()
            .toList();
      })(),
      defaultAccessModeId: (() {
        final value = json['defaultAccessModeId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineCatalogEngine.defaultAccessModeId is required');
        }
        return value;
      })(),
      accessModes: (() {
        final list = _sdkworkAsList(json['accessModes']);
        if (list == null) {
          throw FormatException('AgentEngineCatalogEngine.accessModes is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentEngineAccessModeCatalogEntry.fromJson(map);
      })())
            .whereType<AgentEngineAccessModeCatalogEntry>()
            .toList();
      })(),
      available: (() {
        final value = json['available'];
        if (value is! bool) {
          throw FormatException('AgentEngineCatalogEngine.available is required');
        }
        return value;
      })(),
      unavailableReason: json['unavailableReason']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineKey': engineKey,
      'engineKind': engineKind,
      'tier': tier,
      'agentId': agentId,
      'bindingId': bindingId,
      'models': models.map((item) => item.toJson()).toList(),
      'defaultAccessModeId': defaultAccessModeId,
      'accessModes': accessModes.map((item) => item.toJson()).toList(),
      'available': available,
      'unavailableReason': unavailableReason,
    };
  }
}

class AgentEngineAccessModeCatalogEntry {
  final String modeId;
  final String displayName;
  final String description;
  final String approvalBehavior;
  final String workspaceAccess;
  final String networkAccess;
  final String riskLevel;
  final bool enabled;
  final String? disabledReason;

  AgentEngineAccessModeCatalogEntry({
    required this.modeId,
    required this.displayName,
    required this.description,
    required this.approvalBehavior,
    required this.workspaceAccess,
    required this.networkAccess,
    required this.riskLevel,
    required this.enabled,
    this.disabledReason
  });

  factory AgentEngineAccessModeCatalogEntry.fromJson(Map<String, dynamic> json) {
    return AgentEngineAccessModeCatalogEntry(
      modeId: (() {
        final value = json['modeId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.modeId is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.displayName is required');
        }
        return value;
      })(),
      description: (() {
        final value = json['description']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.description is required');
        }
        return value;
      })(),
      approvalBehavior: (() {
        final value = json['approvalBehavior']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.approvalBehavior is required');
        }
        return value;
      })(),
      workspaceAccess: (() {
        final value = json['workspaceAccess']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.workspaceAccess is required');
        }
        return value;
      })(),
      networkAccess: (() {
        final value = json['networkAccess']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.networkAccess is required');
        }
        return value;
      })(),
      riskLevel: (() {
        final value = json['riskLevel']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.riskLevel is required');
        }
        return value;
      })(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('AgentEngineAccessModeCatalogEntry.enabled is required');
        }
        return value;
      })(),
      disabledReason: json['disabledReason']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'modeId': modeId,
      'displayName': displayName,
      'description': description,
      'approvalBehavior': approvalBehavior,
      'workspaceAccess': workspaceAccess,
      'networkAccess': networkAccess,
      'riskLevel': riskLevel,
      'enabled': enabled,
      'disabledReason': disabledReason,
    };
  }
}

class AgentEngineModelCatalogEntry {
  final String engineKey;
  final String modelId;
  final String label;
  final String description;
  final String providerId;
  final String bindingId;
  final bool defaultForEngine;

  AgentEngineModelCatalogEntry({
    required this.engineKey,
    required this.modelId,
    required this.label,
    required this.description,
    required this.providerId,
    required this.bindingId,
    required this.defaultForEngine
  });

  factory AgentEngineModelCatalogEntry.fromJson(Map<String, dynamic> json) {
    return AgentEngineModelCatalogEntry(
      engineKey: (() {
        final value = json['engineKey']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.engineKey is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.modelId is required');
        }
        return value;
      })(),
      label: (() {
        final value = json['label']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.label is required');
        }
        return value;
      })(),
      description: (() {
        final value = json['description']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.description is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.providerId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentEngineModelCatalogEntry.bindingId is required');
        }
        return value;
      })(),
      defaultForEngine: (() {
        final value = json['defaultForEngine'];
        if (value is! bool) {
          throw FormatException('AgentEngineModelCatalogEntry.defaultForEngine is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineKey': engineKey,
      'modelId': modelId,
      'label': label,
      'description': description,
      'providerId': providerId,
      'bindingId': bindingId,
      'defaultForEngine': defaultForEngine,
    };
  }
}

class McpServerMarketplaceListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  McpServerMarketplaceListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory McpServerMarketplaceListResponse.fromJson(Map<String, dynamic> json) {
    return McpServerMarketplaceListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('McpServerMarketplaceListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('McpServerMarketplaceListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class McpServerMarketplaceRecord {
  final String agentId;
  final String slotId;
  final String serverId;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final bool enabled;
  final int priority;

  McpServerMarketplaceRecord({
    required this.agentId,
    required this.slotId,
    required this.serverId,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.enabled,
    required this.priority
  });

  factory McpServerMarketplaceRecord.fromJson(Map<String, dynamic> json) {
    return McpServerMarketplaceRecord(
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.agentId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.slotId is required');
        }
        return value;
      })(),
      serverId: (() {
        final value = json['serverId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.serverId is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('McpServerMarketplaceRecord.enabled is required');
        }
        return value;
      })(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('McpServerMarketplaceRecord.priority is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentId': agentId,
      'slotId': slotId,
      'serverId': serverId,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'enabled': enabled,
      'priority': priority,
    };
  }
}

class AgentSessionRecord {
  final String id;
  final String sessionId;
  final String tenantId;
  final String organizationId;
  final String agentId;
  final String ownerUserId;
  final String? projectId;
  final String sessionKind;
  final String entrySurface;
  final String? sourceModule;
  final String? sourceContextKind;
  final String? sourceContextId;
  final String? parentSessionId;
  final String? forkedFromTurnId;
  final String? title;
  final String titleSource;
  final String status;
  final String itemCount;
  final String lastItemSequence;
  final String totalInputTokens;
  final String totalOutputTokens;
  final String? idempotencyKey;
  final String? payloadHash;
  final String createdBy;
  final String updatedBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? lastItemAt;
  final String? closedAt;
  final String? archivedAt;
  final String? archivedBy;
  final String? deletedAt;
  final String? deletedBy;
  final String? retentionUntil;

  AgentSessionRecord({
    required this.id,
    required this.sessionId,
    required this.tenantId,
    required this.organizationId,
    required this.agentId,
    required this.ownerUserId,
    this.projectId,
    required this.sessionKind,
    required this.entrySurface,
    this.sourceModule,
    this.sourceContextKind,
    this.sourceContextId,
    this.parentSessionId,
    this.forkedFromTurnId,
    this.title,
    required this.titleSource,
    required this.status,
    required this.itemCount,
    required this.lastItemSequence,
    required this.totalInputTokens,
    required this.totalOutputTokens,
    this.idempotencyKey,
    this.payloadHash,
    required this.createdBy,
    required this.updatedBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.lastItemAt,
    this.closedAt,
    this.archivedAt,
    this.archivedBy,
    this.deletedAt,
    this.deletedBy,
    this.retentionUntil
  });

  factory AgentSessionRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.id is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.sessionId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.organizationId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.agentId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.ownerUserId is required');
        }
        return value;
      })(),
      projectId: json['projectId']?.toString(),
      sessionKind: (() {
        final value = json['sessionKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.sessionKind is required');
        }
        return value;
      })(),
      entrySurface: (() {
        final value = json['entrySurface']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.entrySurface is required');
        }
        return value;
      })(),
      sourceModule: json['sourceModule']?.toString(),
      sourceContextKind: json['sourceContextKind']?.toString(),
      sourceContextId: json['sourceContextId']?.toString(),
      parentSessionId: json['parentSessionId']?.toString(),
      forkedFromTurnId: json['forkedFromTurnId']?.toString(),
      title: json['title']?.toString(),
      titleSource: (() {
        final value = json['titleSource']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.titleSource is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.status is required');
        }
        return value;
      })(),
      itemCount: (() {
        final value = json['itemCount']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.itemCount is required');
        }
        return value;
      })(),
      lastItemSequence: (() {
        final value = json['lastItemSequence']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.lastItemSequence is required');
        }
        return value;
      })(),
      totalInputTokens: (() {
        final value = json['totalInputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.totalInputTokens is required');
        }
        return value;
      })(),
      totalOutputTokens: (() {
        final value = json['totalOutputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.totalOutputTokens is required');
        }
        return value;
      })(),
      idempotencyKey: json['idempotencyKey']?.toString(),
      payloadHash: json['payloadHash']?.toString(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.createdBy is required');
        }
        return value;
      })(),
      updatedBy: (() {
        final value = json['updatedBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.updatedBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.updatedAt is required');
        }
        return value;
      })(),
      lastItemAt: json['lastItemAt']?.toString(),
      closedAt: json['closedAt']?.toString(),
      archivedAt: json['archivedAt']?.toString(),
      archivedBy: json['archivedBy']?.toString(),
      deletedAt: json['deletedAt']?.toString(),
      deletedBy: json['deletedBy']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'sessionId': sessionId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'agentId': agentId,
      'ownerUserId': ownerUserId,
      'projectId': projectId,
      'sessionKind': sessionKind,
      'entrySurface': entrySurface,
      'sourceModule': sourceModule,
      'sourceContextKind': sourceContextKind,
      'sourceContextId': sourceContextId,
      'parentSessionId': parentSessionId,
      'forkedFromTurnId': forkedFromTurnId,
      'title': title,
      'titleSource': titleSource,
      'status': status,
      'itemCount': itemCount,
      'lastItemSequence': lastItemSequence,
      'totalInputTokens': totalInputTokens,
      'totalOutputTokens': totalOutputTokens,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'createdBy': createdBy,
      'updatedBy': updatedBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'lastItemAt': lastItemAt,
      'closedAt': closedAt,
      'archivedAt': archivedAt,
      'archivedBy': archivedBy,
      'deletedAt': deletedAt,
      'deletedBy': deletedBy,
      'retentionUntil': retentionUntil,
    };
  }
}

class SessionProviderIdentity {
  final String? runtimeBindingId;
  final String? providerBindingId;
  final String? providerId;
  final String? modelId;
  final String? providerSessionId;
  final String? providerSessionTreeId;
  final String? providerParentSessionId;
  final String? providerForkedFromSessionId;

  SessionProviderIdentity({
    required this.runtimeBindingId,
    required this.providerBindingId,
    required this.providerId,
    required this.modelId,
    required this.providerSessionId,
    required this.providerSessionTreeId,
    required this.providerParentSessionId,
    required this.providerForkedFromSessionId
  });

  factory SessionProviderIdentity.fromJson(Map<String, dynamic> json) {
    return SessionProviderIdentity(
      runtimeBindingId: (() {
        if (!json.containsKey('runtimeBindingId')) {
          throw FormatException('SessionProviderIdentity.runtimeBindingId is required');
        }
        final _sdkworkRequiredValue = json['runtimeBindingId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.runtimeBindingId is required');
        }
        return value;
      })();
      })(),
      providerBindingId: (() {
        if (!json.containsKey('providerBindingId')) {
          throw FormatException('SessionProviderIdentity.providerBindingId is required');
        }
        final _sdkworkRequiredValue = json['providerBindingId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerBindingId is required');
        }
        return value;
      })();
      })(),
      providerId: (() {
        if (!json.containsKey('providerId')) {
          throw FormatException('SessionProviderIdentity.providerId is required');
        }
        final _sdkworkRequiredValue = json['providerId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerId is required');
        }
        return value;
      })();
      })(),
      modelId: (() {
        if (!json.containsKey('modelId')) {
          throw FormatException('SessionProviderIdentity.modelId is required');
        }
        final _sdkworkRequiredValue = json['modelId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.modelId is required');
        }
        return value;
      })();
      })(),
      providerSessionId: (() {
        if (!json.containsKey('providerSessionId')) {
          throw FormatException('SessionProviderIdentity.providerSessionId is required');
        }
        final _sdkworkRequiredValue = json['providerSessionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerSessionId is required');
        }
        return value;
      })();
      })(),
      providerSessionTreeId: (() {
        if (!json.containsKey('providerSessionTreeId')) {
          throw FormatException('SessionProviderIdentity.providerSessionTreeId is required');
        }
        final _sdkworkRequiredValue = json['providerSessionTreeId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerSessionTreeId is required');
        }
        return value;
      })();
      })(),
      providerParentSessionId: (() {
        if (!json.containsKey('providerParentSessionId')) {
          throw FormatException('SessionProviderIdentity.providerParentSessionId is required');
        }
        final _sdkworkRequiredValue = json['providerParentSessionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerParentSessionId is required');
        }
        return value;
      })();
      })(),
      providerForkedFromSessionId: (() {
        if (!json.containsKey('providerForkedFromSessionId')) {
          throw FormatException('SessionProviderIdentity.providerForkedFromSessionId is required');
        }
        final _sdkworkRequiredValue = json['providerForkedFromSessionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderIdentity.providerForkedFromSessionId is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeBindingId': runtimeBindingId,
      'providerBindingId': providerBindingId,
      'providerId': providerId,
      'modelId': modelId,
      'providerSessionId': providerSessionId,
      'providerSessionTreeId': providerSessionTreeId,
      'providerParentSessionId': providerParentSessionId,
      'providerForkedFromSessionId': providerForkedFromSessionId,
    };
  }
}

class SessionActivityFreshness {
  final String activityAt;
  final String source;
  final String? observedAt;
  final String? freshUntil;
  final String sessionVersion;
  final String? latestTurnVersion;
  final String? latestInteractionId;
  final String? latestInteractionVersion;
  final String? latestRuntimeBindingId;
  final String? latestRuntimeBindingVersion;
  final String? pendingInteractionVersion;
  final String? currentRuntimeBindingVersion;
  final String? userStateVersion;

  SessionActivityFreshness({
    required this.activityAt,
    required this.source,
    required this.observedAt,
    required this.freshUntil,
    required this.sessionVersion,
    required this.latestTurnVersion,
    required this.latestInteractionId,
    required this.latestInteractionVersion,
    required this.latestRuntimeBindingId,
    required this.latestRuntimeBindingVersion,
    required this.pendingInteractionVersion,
    required this.currentRuntimeBindingVersion,
    required this.userStateVersion
  });

  factory SessionActivityFreshness.fromJson(Map<String, dynamic> json) {
    return SessionActivityFreshness(
      activityAt: (() {
        final value = json['activityAt']?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.activityAt is required');
        }
        return value;
      })(),
      source: (() {
        final value = json['source']?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.source is required');
        }
        return value;
      })(),
      observedAt: (() {
        if (!json.containsKey('observedAt')) {
          throw FormatException('SessionActivityFreshness.observedAt is required');
        }
        final _sdkworkRequiredValue = json['observedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.observedAt is required');
        }
        return value;
      })();
      })(),
      freshUntil: (() {
        if (!json.containsKey('freshUntil')) {
          throw FormatException('SessionActivityFreshness.freshUntil is required');
        }
        final _sdkworkRequiredValue = json['freshUntil'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.freshUntil is required');
        }
        return value;
      })();
      })(),
      sessionVersion: (() {
        final value = json['sessionVersion']?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.sessionVersion is required');
        }
        return value;
      })(),
      latestTurnVersion: (() {
        if (!json.containsKey('latestTurnVersion')) {
          throw FormatException('SessionActivityFreshness.latestTurnVersion is required');
        }
        final _sdkworkRequiredValue = json['latestTurnVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.latestTurnVersion is required');
        }
        return value;
      })();
      })(),
      latestInteractionId: (() {
        if (!json.containsKey('latestInteractionId')) {
          throw FormatException('SessionActivityFreshness.latestInteractionId is required');
        }
        final _sdkworkRequiredValue = json['latestInteractionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.latestInteractionId is required');
        }
        return value;
      })();
      })(),
      latestInteractionVersion: (() {
        if (!json.containsKey('latestInteractionVersion')) {
          throw FormatException('SessionActivityFreshness.latestInteractionVersion is required');
        }
        final _sdkworkRequiredValue = json['latestInteractionVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.latestInteractionVersion is required');
        }
        return value;
      })();
      })(),
      latestRuntimeBindingId: (() {
        if (!json.containsKey('latestRuntimeBindingId')) {
          throw FormatException('SessionActivityFreshness.latestRuntimeBindingId is required');
        }
        final _sdkworkRequiredValue = json['latestRuntimeBindingId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.latestRuntimeBindingId is required');
        }
        return value;
      })();
      })(),
      latestRuntimeBindingVersion: (() {
        if (!json.containsKey('latestRuntimeBindingVersion')) {
          throw FormatException('SessionActivityFreshness.latestRuntimeBindingVersion is required');
        }
        final _sdkworkRequiredValue = json['latestRuntimeBindingVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.latestRuntimeBindingVersion is required');
        }
        return value;
      })();
      })(),
      pendingInteractionVersion: (() {
        if (!json.containsKey('pendingInteractionVersion')) {
          throw FormatException('SessionActivityFreshness.pendingInteractionVersion is required');
        }
        final _sdkworkRequiredValue = json['pendingInteractionVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.pendingInteractionVersion is required');
        }
        return value;
      })();
      })(),
      currentRuntimeBindingVersion: (() {
        if (!json.containsKey('currentRuntimeBindingVersion')) {
          throw FormatException('SessionActivityFreshness.currentRuntimeBindingVersion is required');
        }
        final _sdkworkRequiredValue = json['currentRuntimeBindingVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.currentRuntimeBindingVersion is required');
        }
        return value;
      })();
      })(),
      userStateVersion: (() {
        if (!json.containsKey('userStateVersion')) {
          throw FormatException('SessionActivityFreshness.userStateVersion is required');
        }
        final _sdkworkRequiredValue = json['userStateVersion'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionActivityFreshness.userStateVersion is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'activityAt': activityAt,
      'source': source,
      'observedAt': observedAt,
      'freshUntil': freshUntil,
      'sessionVersion': sessionVersion,
      'latestTurnVersion': latestTurnVersion,
      'latestInteractionId': latestInteractionId,
      'latestInteractionVersion': latestInteractionVersion,
      'latestRuntimeBindingId': latestRuntimeBindingId,
      'latestRuntimeBindingVersion': latestRuntimeBindingVersion,
      'pendingInteractionVersion': pendingInteractionVersion,
      'currentRuntimeBindingVersion': currentRuntimeBindingVersion,
      'userStateVersion': userStateVersion,
    };
  }
}

class SessionProviderActivityObservation {
  final String providerSessionId;
  final String? state;
  final String freshness;
  final String? evidenceKind;
  final String? interactionHint;
  final String? observedAt;
  final String? freshUntil;

  SessionProviderActivityObservation({
    required this.providerSessionId,
    required this.state,
    required this.freshness,
    required this.evidenceKind,
    required this.interactionHint,
    required this.observedAt,
    required this.freshUntil
  });

  factory SessionProviderActivityObservation.fromJson(Map<String, dynamic> json) {
    return SessionProviderActivityObservation(
      providerSessionId: (() {
        final value = json['providerSessionId']?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.providerSessionId is required');
        }
        return value;
      })(),
      state: (() {
        if (!json.containsKey('state')) {
          throw FormatException('SessionProviderActivityObservation.state is required');
        }
        final _sdkworkRequiredValue = json['state'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.state is required');
        }
        return value;
      })();
      })(),
      freshness: (() {
        final value = json['freshness']?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.freshness is required');
        }
        return value;
      })(),
      evidenceKind: (() {
        if (!json.containsKey('evidenceKind')) {
          throw FormatException('SessionProviderActivityObservation.evidenceKind is required');
        }
        final _sdkworkRequiredValue = json['evidenceKind'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.evidenceKind is required');
        }
        return value;
      })();
      })(),
      interactionHint: (() {
        if (!json.containsKey('interactionHint')) {
          throw FormatException('SessionProviderActivityObservation.interactionHint is required');
        }
        final _sdkworkRequiredValue = json['interactionHint'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.interactionHint is required');
        }
        return value;
      })();
      })(),
      observedAt: (() {
        if (!json.containsKey('observedAt')) {
          throw FormatException('SessionProviderActivityObservation.observedAt is required');
        }
        final _sdkworkRequiredValue = json['observedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.observedAt is required');
        }
        return value;
      })();
      })(),
      freshUntil: (() {
        if (!json.containsKey('freshUntil')) {
          throw FormatException('SessionProviderActivityObservation.freshUntil is required');
        }
        final _sdkworkRequiredValue = json['freshUntil'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('SessionProviderActivityObservation.freshUntil is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'providerSessionId': providerSessionId,
      'state': state,
      'freshness': freshness,
      'evidenceKind': evidenceKind,
      'interactionHint': interactionHint,
      'observedAt': observedAt,
      'freshUntil': freshUntil,
    };
  }
}

class SessionActivitySummary {
  final AgentSessionRecord session;
  final AgentTurnRecord? latestTurn;
  final AgentInteractionRecord? pendingInteraction;
  final AgentSessionRuntimeBindingRecord? currentRuntimeBinding;
  final AgentSessionRuntimeBindingRecord? latestRuntimeBinding;
  final AgentResourceUserStateRecord? userState;
  final SessionProviderIdentity providerIdentity;
  final SessionActivityFreshness freshness;
  final SessionProviderActivityObservation? providerActivity;
  final String presentationPhase;

  SessionActivitySummary({
    required this.session,
    required this.latestTurn,
    required this.pendingInteraction,
    required this.currentRuntimeBinding,
    required this.latestRuntimeBinding,
    required this.userState,
    required this.providerIdentity,
    required this.freshness,
    required this.providerActivity,
    required this.presentationPhase
  });

  factory SessionActivitySummary.fromJson(Map<String, dynamic> json) {
    return SessionActivitySummary(
      session: (() {
        final map = _sdkworkAsMap(json['session']);
        if (map == null) {
          throw FormatException('SessionActivitySummary.session is required');
        }
        return AgentSessionRecord.fromJson(map);
      })(),
      latestTurn: (() {
        if (!json.containsKey('latestTurn')) {
          throw FormatException('SessionActivitySummary.latestTurn is required');
        }
        final _sdkworkRequiredValue = json['latestTurn'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.latestTurn is required');
        }
        return AgentTurnRecord.fromJson(map);
      })();
      })(),
      pendingInteraction: (() {
        if (!json.containsKey('pendingInteraction')) {
          throw FormatException('SessionActivitySummary.pendingInteraction is required');
        }
        final _sdkworkRequiredValue = json['pendingInteraction'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.pendingInteraction is required');
        }
        return AgentInteractionRecord.fromJson(map);
      })();
      })(),
      currentRuntimeBinding: (() {
        if (!json.containsKey('currentRuntimeBinding')) {
          throw FormatException('SessionActivitySummary.currentRuntimeBinding is required');
        }
        final _sdkworkRequiredValue = json['currentRuntimeBinding'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.currentRuntimeBinding is required');
        }
        return AgentSessionRuntimeBindingRecord.fromJson(map);
      })();
      })(),
      latestRuntimeBinding: (() {
        if (!json.containsKey('latestRuntimeBinding')) {
          throw FormatException('SessionActivitySummary.latestRuntimeBinding is required');
        }
        final _sdkworkRequiredValue = json['latestRuntimeBinding'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.latestRuntimeBinding is required');
        }
        return AgentSessionRuntimeBindingRecord.fromJson(map);
      })();
      })(),
      userState: (() {
        if (!json.containsKey('userState')) {
          throw FormatException('SessionActivitySummary.userState is required');
        }
        final _sdkworkRequiredValue = json['userState'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.userState is required');
        }
        return AgentResourceUserStateRecord.fromJson(map);
      })();
      })(),
      providerIdentity: (() {
        final map = _sdkworkAsMap(json['providerIdentity']);
        if (map == null) {
          throw FormatException('SessionActivitySummary.providerIdentity is required');
        }
        return SessionProviderIdentity.fromJson(map);
      })(),
      freshness: (() {
        final map = _sdkworkAsMap(json['freshness']);
        if (map == null) {
          throw FormatException('SessionActivitySummary.freshness is required');
        }
        return SessionActivityFreshness.fromJson(map);
      })(),
      providerActivity: (() {
        if (!json.containsKey('providerActivity')) {
          throw FormatException('SessionActivitySummary.providerActivity is required');
        }
        final _sdkworkRequiredValue = json['providerActivity'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('SessionActivitySummary.providerActivity is required');
        }
        return SessionProviderActivityObservation.fromJson(map);
      })();
      })(),
      presentationPhase: (() {
        final value = json['presentationPhase']?.toString();
        if (value == null) {
          throw FormatException('SessionActivitySummary.presentationPhase is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'session': session.toJson(),
      'latestTurn': latestTurn?.toJson(),
      'pendingInteraction': pendingInteraction?.toJson(),
      'currentRuntimeBinding': currentRuntimeBinding?.toJson(),
      'latestRuntimeBinding': latestRuntimeBinding?.toJson(),
      'userState': userState?.toJson(),
      'providerIdentity': providerIdentity.toJson(),
      'freshness': freshness.toJson(),
      'providerActivity': providerActivity?.toJson(),
      'presentationPhase': presentationPhase,
    };
  }
}

class SessionActivitySummaryListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  SessionActivitySummaryListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory SessionActivitySummaryListResponse.fromJson(Map<String, dynamic> json) {
    return SessionActivitySummaryListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('SessionActivitySummaryListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('SessionActivitySummaryListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('SessionActivitySummaryListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ProjectSessionSynchronizationIssue {
  final String code;
  final String count;
  final String disposition;

  ProjectSessionSynchronizationIssue({
    required this.code,
    required this.count,
    required this.disposition
  });

  factory ProjectSessionSynchronizationIssue.fromJson(Map<String, dynamic> json) {
    return ProjectSessionSynchronizationIssue(
      code: (() {
        final value = json['code']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationIssue.code is required');
        }
        return value;
      })(),
      count: (() {
        final value = json['count']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationIssue.count is required');
        }
        return value;
      })(),
      disposition: (() {
        final value = json['disposition']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationIssue.disposition is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'count': count,
      'disposition': disposition,
    };
  }
}

class ProjectSessionSynchronizationResult {
  final String status;
  final String projectId;
  final String synchronizedSessionCount;
  final String skippedSessionCount;
  final String failedSessionCount;
  final List<ProjectSessionSynchronizationIssue> issues;

  ProjectSessionSynchronizationResult({
    required this.status,
    required this.projectId,
    required this.synchronizedSessionCount,
    required this.skippedSessionCount,
    required this.failedSessionCount,
    required this.issues
  });

  factory ProjectSessionSynchronizationResult.fromJson(Map<String, dynamic> json) {
    return ProjectSessionSynchronizationResult(
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResult.status is required');
        }
        return value;
      })(),
      projectId: (() {
        final value = json['projectId']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResult.projectId is required');
        }
        return value;
      })(),
      synchronizedSessionCount: (() {
        final value = json['synchronizedSessionCount']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResult.synchronizedSessionCount is required');
        }
        return value;
      })(),
      skippedSessionCount: (() {
        final value = json['skippedSessionCount']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResult.skippedSessionCount is required');
        }
        return value;
      })(),
      failedSessionCount: (() {
        final value = json['failedSessionCount']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResult.failedSessionCount is required');
        }
        return value;
      })(),
      issues: (() {
        final list = _sdkworkAsList(json['issues']);
        if (list == null) {
          throw FormatException('ProjectSessionSynchronizationResult.issues is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : ProjectSessionSynchronizationIssue.fromJson(map);
      })())
            .whereType<ProjectSessionSynchronizationIssue>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'status': status,
      'projectId': projectId,
      'synchronizedSessionCount': synchronizedSessionCount,
      'skippedSessionCount': skippedSessionCount,
      'failedSessionCount': failedSessionCount,
      'issues': issues.map((item) => item.toJson()).toList(),
    };
  }
}

class ProjectSessionSynchronizationResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ProjectSessionSynchronizationResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ProjectSessionSynchronizationResponse.fromJson(Map<String, dynamic> json) {
    return ProjectSessionSynchronizationResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ProjectSessionSynchronizationResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ProjectSessionSynchronizationResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ProjectSessionSynchronizationResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionRequest {
  final String? agentId;
  final String? projectId;
  final String sessionKind;
  final String entrySurface;
  final String? sourceModule;
  final String? sourceContextKind;
  final String? sourceContextId;
  final String? parentSessionId;
  final String? forkedFromTurnId;
  final String? title;
  final String idempotencyKey;
  final String payloadHash;
  final String requestedAt;

  CreateAgentSessionRequest({
    this.agentId,
    this.projectId,
    required this.sessionKind,
    required this.entrySurface,
    this.sourceModule,
    this.sourceContextKind,
    this.sourceContextId,
    this.parentSessionId,
    this.forkedFromTurnId,
    this.title,
    required this.idempotencyKey,
    required this.payloadHash,
    required this.requestedAt
  });

  factory CreateAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionRequest(
      agentId: json['agentId']?.toString(),
      projectId: json['projectId']?.toString(),
      sessionKind: (() {
        final value = json['sessionKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.sessionKind is required');
        }
        return value;
      })(),
      entrySurface: (() {
        final value = json['entrySurface']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.entrySurface is required');
        }
        return value;
      })(),
      sourceModule: json['sourceModule']?.toString(),
      sourceContextKind: json['sourceContextKind']?.toString(),
      sourceContextId: json['sourceContextId']?.toString(),
      parentSessionId: json['parentSessionId']?.toString(),
      forkedFromTurnId: json['forkedFromTurnId']?.toString(),
      title: json['title']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.payloadHash is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentId': agentId,
      'projectId': projectId,
      'sessionKind': sessionKind,
      'entrySurface': entrySurface,
      'sourceModule': sourceModule,
      'sourceContextKind': sourceContextKind,
      'sourceContextId': sourceContextId,
      'parentSessionId': parentSessionId,
      'forkedFromTurnId': forkedFromTurnId,
      'title': title,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'requestedAt': requestedAt,
    };
  }
}

class ExecuteAgentTaskRequest {
  final String idempotencyKey;
  final String? expectedVersion;
  final String requestedAt;

  ExecuteAgentTaskRequest({
    required this.idempotencyKey,
    this.expectedVersion,
    required this.requestedAt
  });

  factory ExecuteAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return ExecuteAgentTaskRequest(
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('ExecuteAgentTaskRequest.idempotencyKey is required');
        }
        return value;
      })(),
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ExecuteAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'idempotencyKey': idempotencyKey,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class RetryAgentTaskRunRequest {
  final String idempotencyKey;
  final String requestedAt;

  RetryAgentTaskRunRequest({
    required this.idempotencyKey,
    required this.requestedAt
  });

  factory RetryAgentTaskRunRequest.fromJson(Map<String, dynamic> json) {
    return RetryAgentTaskRunRequest(
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('RetryAgentTaskRunRequest.idempotencyKey is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RetryAgentTaskRunRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'idempotencyKey': idempotencyKey,
      'requestedAt': requestedAt,
    };
  }
}

class CancelAgentTaskRunRequest {
  final String? expectedVersion;
  final String requestedAt;

  CancelAgentTaskRunRequest({
    this.expectedVersion,
    required this.requestedAt
  });

  factory CancelAgentTaskRunRequest.fromJson(Map<String, dynamic> json) {
    return CancelAgentTaskRunRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTaskRunRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentTaskRunRecord {
  final String runId;
  final String taskId;
  final String sessionId;
  final String agentId;
  final String ownerUserId;
  final String triggerKind;
  final String scheduleGeneration;
  final String scheduledFor;
  final String? retryOfRunId;
  final int priority;
  final String status;
  final String? turnId;
  final int attemptCount;
  final int maxAttempts;
  final String availableAt;
  final String? timeoutAt;
  final String? failureClass;
  final String? errorCode;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? claimedAt;
  final String? startedAt;
  final String? finishedAt;
  final String? cancelledAt;

  AgentTaskRunRecord({
    required this.runId,
    required this.taskId,
    required this.sessionId,
    required this.agentId,
    required this.ownerUserId,
    required this.triggerKind,
    required this.scheduleGeneration,
    required this.scheduledFor,
    required this.retryOfRunId,
    required this.priority,
    required this.status,
    required this.turnId,
    required this.attemptCount,
    required this.maxAttempts,
    required this.availableAt,
    required this.timeoutAt,
    required this.failureClass,
    required this.errorCode,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    required this.claimedAt,
    required this.startedAt,
    required this.finishedAt,
    required this.cancelledAt
  });

  factory AgentTaskRunRecord.fromJson(Map<String, dynamic> json) {
    return AgentTaskRunRecord(
      runId: (() {
        final value = json['runId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.runId is required');
        }
        return value;
      })(),
      taskId: (() {
        final value = json['taskId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.taskId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.sessionId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.agentId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.ownerUserId is required');
        }
        return value;
      })(),
      triggerKind: (() {
        final value = json['triggerKind']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.triggerKind is required');
        }
        return value;
      })(),
      scheduleGeneration: (() {
        final value = json['scheduleGeneration']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.scheduleGeneration is required');
        }
        return value;
      })(),
      scheduledFor: (() {
        final value = json['scheduledFor']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.scheduledFor is required');
        }
        return value;
      })(),
      retryOfRunId: (() {
        if (!json.containsKey('retryOfRunId')) {
          throw FormatException('AgentTaskRunRecord.retryOfRunId is required');
        }
        final _sdkworkRequiredValue = json['retryOfRunId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.retryOfRunId is required');
        }
        return value;
      })();
      })(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentTaskRunRecord.priority is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.status is required');
        }
        return value;
      })(),
      turnId: (() {
        if (!json.containsKey('turnId')) {
          throw FormatException('AgentTaskRunRecord.turnId is required');
        }
        final _sdkworkRequiredValue = json['turnId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.turnId is required');
        }
        return value;
      })();
      })(),
      attemptCount: (() {
        final value = json['attemptCount'];
        if (value is! int) {
          throw FormatException('AgentTaskRunRecord.attemptCount is required');
        }
        return value;
      })(),
      maxAttempts: (() {
        final value = json['maxAttempts'];
        if (value is! int) {
          throw FormatException('AgentTaskRunRecord.maxAttempts is required');
        }
        return value;
      })(),
      availableAt: (() {
        final value = json['availableAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.availableAt is required');
        }
        return value;
      })(),
      timeoutAt: (() {
        if (!json.containsKey('timeoutAt')) {
          throw FormatException('AgentTaskRunRecord.timeoutAt is required');
        }
        final _sdkworkRequiredValue = json['timeoutAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.timeoutAt is required');
        }
        return value;
      })();
      })(),
      failureClass: (() {
        if (!json.containsKey('failureClass')) {
          throw FormatException('AgentTaskRunRecord.failureClass is required');
        }
        final _sdkworkRequiredValue = json['failureClass'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.failureClass is required');
        }
        return value;
      })();
      })(),
      errorCode: (() {
        if (!json.containsKey('errorCode')) {
          throw FormatException('AgentTaskRunRecord.errorCode is required');
        }
        final _sdkworkRequiredValue = json['errorCode'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.errorCode is required');
        }
        return value;
      })();
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.updatedAt is required');
        }
        return value;
      })(),
      claimedAt: (() {
        if (!json.containsKey('claimedAt')) {
          throw FormatException('AgentTaskRunRecord.claimedAt is required');
        }
        final _sdkworkRequiredValue = json['claimedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.claimedAt is required');
        }
        return value;
      })();
      })(),
      startedAt: (() {
        if (!json.containsKey('startedAt')) {
          throw FormatException('AgentTaskRunRecord.startedAt is required');
        }
        final _sdkworkRequiredValue = json['startedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.startedAt is required');
        }
        return value;
      })();
      })(),
      finishedAt: (() {
        if (!json.containsKey('finishedAt')) {
          throw FormatException('AgentTaskRunRecord.finishedAt is required');
        }
        final _sdkworkRequiredValue = json['finishedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.finishedAt is required');
        }
        return value;
      })();
      })(),
      cancelledAt: (() {
        if (!json.containsKey('cancelledAt')) {
          throw FormatException('AgentTaskRunRecord.cancelledAt is required');
        }
        final _sdkworkRequiredValue = json['cancelledAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunRecord.cancelledAt is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runId': runId,
      'taskId': taskId,
      'sessionId': sessionId,
      'agentId': agentId,
      'ownerUserId': ownerUserId,
      'triggerKind': triggerKind,
      'scheduleGeneration': scheduleGeneration,
      'scheduledFor': scheduledFor,
      'retryOfRunId': retryOfRunId,
      'priority': priority,
      'status': status,
      'turnId': turnId,
      'attemptCount': attemptCount,
      'maxAttempts': maxAttempts,
      'availableAt': availableAt,
      'timeoutAt': timeoutAt,
      'failureClass': failureClass,
      'errorCode': errorCode,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'claimedAt': claimedAt,
      'startedAt': startedAt,
      'finishedAt': finishedAt,
      'cancelledAt': cancelledAt,
    };
  }
}

class AgentTaskRunAttemptRecord {
  final String attemptId;
  final String runId;
  final int attemptNo;
  final String status;
  final String? failureClass;
  final String? errorCode;
  final String createdAt;
  final String updatedAt;
  final String? startedAt;
  final String? heartbeatAt;
  final String? finishedAt;

  AgentTaskRunAttemptRecord({
    required this.attemptId,
    required this.runId,
    required this.attemptNo,
    required this.status,
    required this.failureClass,
    required this.errorCode,
    required this.createdAt,
    required this.updatedAt,
    required this.startedAt,
    required this.heartbeatAt,
    required this.finishedAt
  });

  factory AgentTaskRunAttemptRecord.fromJson(Map<String, dynamic> json) {
    return AgentTaskRunAttemptRecord(
      attemptId: (() {
        final value = json['attemptId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.attemptId is required');
        }
        return value;
      })(),
      runId: (() {
        final value = json['runId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.runId is required');
        }
        return value;
      })(),
      attemptNo: (() {
        final value = json['attemptNo'];
        if (value is! int) {
          throw FormatException('AgentTaskRunAttemptRecord.attemptNo is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.status is required');
        }
        return value;
      })(),
      failureClass: (() {
        if (!json.containsKey('failureClass')) {
          throw FormatException('AgentTaskRunAttemptRecord.failureClass is required');
        }
        final _sdkworkRequiredValue = json['failureClass'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.failureClass is required');
        }
        return value;
      })();
      })(),
      errorCode: (() {
        if (!json.containsKey('errorCode')) {
          throw FormatException('AgentTaskRunAttemptRecord.errorCode is required');
        }
        final _sdkworkRequiredValue = json['errorCode'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.errorCode is required');
        }
        return value;
      })();
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.updatedAt is required');
        }
        return value;
      })(),
      startedAt: (() {
        if (!json.containsKey('startedAt')) {
          throw FormatException('AgentTaskRunAttemptRecord.startedAt is required');
        }
        final _sdkworkRequiredValue = json['startedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.startedAt is required');
        }
        return value;
      })();
      })(),
      heartbeatAt: (() {
        if (!json.containsKey('heartbeatAt')) {
          throw FormatException('AgentTaskRunAttemptRecord.heartbeatAt is required');
        }
        final _sdkworkRequiredValue = json['heartbeatAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.heartbeatAt is required');
        }
        return value;
      })();
      })(),
      finishedAt: (() {
        if (!json.containsKey('finishedAt')) {
          throw FormatException('AgentTaskRunAttemptRecord.finishedAt is required');
        }
        final _sdkworkRequiredValue = json['finishedAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptRecord.finishedAt is required');
        }
        return value;
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'attemptId': attemptId,
      'runId': runId,
      'attemptNo': attemptNo,
      'status': status,
      'failureClass': failureClass,
      'errorCode': errorCode,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'startedAt': startedAt,
      'heartbeatAt': heartbeatAt,
      'finishedAt': finishedAt,
    };
  }
}

class AgentTaskRunResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskRunResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskRunResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskRunResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskRunResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskRunResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTaskRunListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskRunListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskRunListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskRunListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskRunListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskRunListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTaskRunAttemptListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskRunAttemptListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskRunAttemptListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskRunAttemptListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskRunAttemptListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskRunAttemptListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRunAttemptListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CloseAgentSessionRequest {
  final String expectedVersion;
  final String requestedAt;

  CloseAgentSessionRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory CloseAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return CloseAgentSessionRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('CloseAgentSessionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CloseAgentSessionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentItemDriveRefRecord {
  final String resourceRole;
  final String driveSpaceId;
  final String driveNodeId;
  final String? mediaResourceId;
  final String? objectBlobId;
  final String? resourceHash;
  final String? altText;
  final int sortOrder;
  final String status;
  final String createdBy;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;
  final String? retentionUntil;

  AgentItemDriveRefRecord({
    required this.resourceRole,
    required this.driveSpaceId,
    required this.driveNodeId,
    this.mediaResourceId,
    this.objectBlobId,
    this.resourceHash,
    this.altText,
    required this.sortOrder,
    required this.status,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt,
    this.retentionUntil
  });

  factory AgentItemDriveRefRecord.fromJson(Map<String, dynamic> json) {
    return AgentItemDriveRefRecord(
      resourceRole: (() {
        final value = json['resourceRole']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.resourceRole is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.driveSpaceId is required');
        }
        return value;
      })(),
      driveNodeId: (() {
        final value = json['driveNodeId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.driveNodeId is required');
        }
        return value;
      })(),
      mediaResourceId: json['mediaResourceId']?.toString(),
      objectBlobId: json['objectBlobId']?.toString(),
      resourceHash: json['resourceHash']?.toString(),
      altText: json['altText']?.toString(),
      sortOrder: (() {
        final value = json['sortOrder'];
        if (value is! int) {
          throw FormatException('AgentItemDriveRefRecord.sortOrder is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.status is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.createdBy is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'resourceRole': resourceRole,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'mediaResourceId': mediaResourceId,
      'objectBlobId': objectBlobId,
      'resourceHash': resourceHash,
      'altText': altText,
      'sortOrder': sortOrder,
      'status': status,
      'createdBy': createdBy,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionItemRecord {
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String itemId;
  final String kind;
  final String? content;
  final String? contentType;
  final String status;
  final String sequence;
  final String inputTokens;
  final String outputTokens;
  final String? modelId;
  final String? providerId;
  final String? toolName;
  final String? toolCallId;
  final Map<String, dynamic>? toolArguments;
  final Map<String, dynamic>? toolResult;
  final Map<String, dynamic>? providerPayload;
  final String? parentItemId;
  final String? turnId;
  final List<AgentItemDriveRefRecord> driveRefs;
  final String createdBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? completedAt;
  final String? redactedAt;
  final String? redactedBy;
  final String? retentionUntil;

  AgentSessionItemRecord({
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    required this.itemId,
    required this.kind,
    this.content,
    this.contentType,
    required this.status,
    required this.sequence,
    required this.inputTokens,
    required this.outputTokens,
    this.modelId,
    this.providerId,
    this.toolName,
    this.toolCallId,
    this.toolArguments,
    this.toolResult,
    this.providerPayload,
    this.parentItemId,
    this.turnId,
    required this.driveRefs,
    required this.createdBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.completedAt,
    this.redactedAt,
    this.redactedBy,
    this.retentionUntil
  });

  factory AgentSessionItemRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.sessionId is required');
        }
        return value;
      })(),
      itemId: (() {
        final value = json['itemId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.itemId is required');
        }
        return value;
      })(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.kind is required');
        }
        return value;
      })(),
      content: json['content']?.toString(),
      contentType: json['contentType']?.toString(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.status is required');
        }
        return value;
      })(),
      sequence: (() {
        final value = json['sequence']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.sequence is required');
        }
        return value;
      })(),
      inputTokens: (() {
        final value = json['inputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.inputTokens is required');
        }
        return value;
      })(),
      outputTokens: (() {
        final value = json['outputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.outputTokens is required');
        }
        return value;
      })(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      toolName: json['toolName']?.toString(),
      toolCallId: json['toolCallId']?.toString(),
      toolArguments: _sdkworkAsMap(json['toolArguments']),
      toolResult: _sdkworkAsMap(json['toolResult']),
      providerPayload: _sdkworkAsMap(json['providerPayload']),
      parentItemId: json['parentItemId']?.toString(),
      turnId: json['turnId']?.toString(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          throw FormatException('AgentSessionItemRecord.driveRefs is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentItemDriveRefRecord.fromJson(map);
      })())
            .whereType<AgentItemDriveRefRecord>()
            .toList();
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.createdBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.updatedAt is required');
        }
        return value;
      })(),
      completedAt: json['completedAt']?.toString(),
      redactedAt: json['redactedAt']?.toString(),
      redactedBy: json['redactedBy']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'itemId': itemId,
      'kind': kind,
      'content': content,
      'contentType': contentType,
      'status': status,
      'sequence': sequence,
      'inputTokens': inputTokens,
      'outputTokens': outputTokens,
      'modelId': modelId,
      'providerId': providerId,
      'toolName': toolName,
      'toolCallId': toolCallId,
      'toolArguments': toolArguments,
      'toolResult': toolResult,
      'providerPayload': providerPayload,
      'parentItemId': parentItemId,
      'turnId': turnId,
      'driveRefs': driveRefs.map((item) => item.toJson()).toList(),
      'createdBy': createdBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'completedAt': completedAt,
      'redactedAt': redactedAt,
      'redactedBy': redactedBy,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionItemResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionItemResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionItemResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionItemResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionItemResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionItemListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionItemListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionItemListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionItemListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionItemListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionItemSynchronizeResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionItemSynchronizeResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionItemSynchronizeResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemSynchronizeResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionItemSynchronizeResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionItemSynchronizeResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemSynchronizeResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionItemSynchronizationResult {
  final String status;
  final String importedItemCount;

  AgentSessionItemSynchronizationResult({
    required this.status,
    required this.importedItemCount
  });

  factory AgentSessionItemSynchronizationResult.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemSynchronizationResult(
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemSynchronizationResult.status is required');
        }
        return value;
      })(),
      importedItemCount: (() {
        final value = json['importedItemCount']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemSynchronizationResult.importedItemCount is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'status': status,
      'importedItemCount': importedItemCount,
    };
  }
}

class AgentTurnInputQueueDriveRef {
  final String resourceRole;
  final String driveSpaceId;
  final String driveNodeId;

  AgentTurnInputQueueDriveRef({
    required this.resourceRole,
    required this.driveSpaceId,
    required this.driveNodeId
  });

  factory AgentTurnInputQueueDriveRef.fromJson(Map<String, dynamic> json) {
    return AgentTurnInputQueueDriveRef(
      resourceRole: (() {
        final value = json['resourceRole']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueDriveRef.resourceRole is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueDriveRef.driveSpaceId is required');
        }
        return value;
      })(),
      driveNodeId: (() {
        final value = json['driveNodeId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueDriveRef.driveNodeId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'resourceRole': resourceRole,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
    };
  }
}

class AgentTurnInputQueueEntry {
  final String queueEntryId;
  final String sessionId;
  final String agentId;
  final String content;
  final String displayText;
  final String contentType;
  final List<String> attachmentNames;
  final List<AgentTurnInputQueueDriveRef> driveRefs;
  final String turnMode;
  final String? systemPrompt;
  final String? runtimeBindingId;
  final String? requestedModelId;
  final String? accessModeId;
  final String idempotencyKey;
  final String payloadHash;
  final String clientRequestId;
  final String position;
  final String status;
  final String? claimOwner;
  final String? claimExpiresAt;
  final String fencingToken;
  final String? errorCode;
  final String? errorDetail;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? claimedAt;
  final String? failedAt;

  AgentTurnInputQueueEntry({
    required this.queueEntryId,
    required this.sessionId,
    required this.agentId,
    required this.content,
    required this.displayText,
    required this.contentType,
    required this.attachmentNames,
    required this.driveRefs,
    required this.turnMode,
    this.systemPrompt,
    this.runtimeBindingId,
    this.requestedModelId,
    this.accessModeId,
    required this.idempotencyKey,
    required this.payloadHash,
    required this.clientRequestId,
    required this.position,
    required this.status,
    this.claimOwner,
    this.claimExpiresAt,
    required this.fencingToken,
    this.errorCode,
    this.errorDetail,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.claimedAt,
    this.failedAt
  });

  factory AgentTurnInputQueueEntry.fromJson(Map<String, dynamic> json) {
    return AgentTurnInputQueueEntry(
      queueEntryId: (() {
        final value = json['queueEntryId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.queueEntryId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.sessionId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.agentId is required');
        }
        return value;
      })(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.content is required');
        }
        return value;
      })(),
      displayText: (() {
        final value = json['displayText']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.displayText is required');
        }
        return value;
      })(),
      contentType: (() {
        final value = json['contentType']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.contentType is required');
        }
        return value;
      })(),
      attachmentNames: (() {
        final list = _sdkworkAsList(json['attachmentNames']);
        if (list == null) {
          throw FormatException('AgentTurnInputQueueEntry.attachmentNames is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          throw FormatException('AgentTurnInputQueueEntry.driveRefs is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentTurnInputQueueDriveRef.fromJson(map);
      })())
            .whereType<AgentTurnInputQueueDriveRef>()
            .toList();
      })(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.turnMode is required');
        }
        return value;
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      requestedModelId: json['requestedModelId']?.toString(),
      accessModeId: json['accessModeId']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.payloadHash is required');
        }
        return value;
      })(),
      clientRequestId: (() {
        final value = json['clientRequestId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.clientRequestId is required');
        }
        return value;
      })(),
      position: (() {
        final value = json['position']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.position is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.status is required');
        }
        return value;
      })(),
      claimOwner: json['claimOwner']?.toString(),
      claimExpiresAt: json['claimExpiresAt']?.toString(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.fencingToken is required');
        }
        return value;
      })(),
      errorCode: json['errorCode']?.toString(),
      errorDetail: json['errorDetail']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntry.updatedAt is required');
        }
        return value;
      })(),
      claimedAt: json['claimedAt']?.toString(),
      failedAt: json['failedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'queueEntryId': queueEntryId,
      'sessionId': sessionId,
      'agentId': agentId,
      'content': content,
      'displayText': displayText,
      'contentType': contentType,
      'attachmentNames': attachmentNames.map((item) => item).toList(),
      'driveRefs': driveRefs.map((item) => item.toJson()).toList(),
      'turnMode': turnMode,
      'systemPrompt': systemPrompt,
      'runtimeBindingId': runtimeBindingId,
      'requestedModelId': requestedModelId,
      'accessModeId': accessModeId,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'clientRequestId': clientRequestId,
      'position': position,
      'status': status,
      'claimOwner': claimOwner,
      'claimExpiresAt': claimExpiresAt,
      'fencingToken': fencingToken,
      'errorCode': errorCode,
      'errorDetail': errorDetail,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'claimedAt': claimedAt,
      'failedAt': failedAt,
    };
  }
}

class CreateAgentTurnInputQueueEntryRequest {
  final String? queueEntryId;
  final String content;
  final String? displayText;
  final String? contentType;
  final List<String>? attachmentNames;
  final List<AgentTurnInputQueueDriveRef>? driveRefs;
  final String turnMode;
  final String? systemPrompt;
  final String? runtimeBindingId;
  final String? requestedModelId;
  final String? accessModeId;
  final String requestedAt;

  CreateAgentTurnInputQueueEntryRequest({
    this.queueEntryId,
    required this.content,
    this.displayText,
    this.contentType,
    this.attachmentNames,
    this.driveRefs,
    required this.turnMode,
    this.systemPrompt,
    this.runtimeBindingId,
    this.requestedModelId,
    this.accessModeId,
    required this.requestedAt
  });

  factory CreateAgentTurnInputQueueEntryRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentTurnInputQueueEntryRequest(
      queueEntryId: json['queueEntryId']?.toString(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnInputQueueEntryRequest.content is required');
        }
        return value;
      })(),
      displayText: json['displayText']?.toString(),
      contentType: json['contentType']?.toString(),
      attachmentNames: (() {
        final list = _sdkworkAsList(json['attachmentNames']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentTurnInputQueueDriveRef.fromJson(map);
      })())
            .whereType<AgentTurnInputQueueDriveRef>()
            .toList();
      })(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnInputQueueEntryRequest.turnMode is required');
        }
        return value;
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      requestedModelId: json['requestedModelId']?.toString(),
      accessModeId: json['accessModeId']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnInputQueueEntryRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'queueEntryId': queueEntryId,
      'content': content,
      'displayText': displayText,
      'contentType': contentType,
      'attachmentNames': attachmentNames?.map((item) => item).toList(),
      'driveRefs': driveRefs?.map((item) => item.toJson()).toList(),
      'turnMode': turnMode,
      'systemPrompt': systemPrompt,
      'runtimeBindingId': runtimeBindingId,
      'requestedModelId': requestedModelId,
      'accessModeId': accessModeId,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentTurnInputQueueEntryRequest {
  final String content;
  final String? displayText;
  final String? contentType;
  final List<String>? attachmentNames;
  final List<AgentTurnInputQueueDriveRef>? driveRefs;
  final String turnMode;
  final String? systemPrompt;
  final String? runtimeBindingId;
  final String? requestedModelId;
  final String? accessModeId;
  final String expectedVersion;
  final String requestedAt;

  UpdateAgentTurnInputQueueEntryRequest({
    required this.content,
    this.displayText,
    this.contentType,
    this.attachmentNames,
    this.driveRefs,
    required this.turnMode,
    this.systemPrompt,
    this.runtimeBindingId,
    this.requestedModelId,
    this.accessModeId,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory UpdateAgentTurnInputQueueEntryRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentTurnInputQueueEntryRequest(
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentTurnInputQueueEntryRequest.content is required');
        }
        return value;
      })(),
      displayText: json['displayText']?.toString(),
      contentType: json['contentType']?.toString(),
      attachmentNames: (() {
        final list = _sdkworkAsList(json['attachmentNames']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentTurnInputQueueDriveRef.fromJson(map);
      })())
            .whereType<AgentTurnInputQueueDriveRef>()
            .toList();
      })(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentTurnInputQueueEntryRequest.turnMode is required');
        }
        return value;
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      requestedModelId: json['requestedModelId']?.toString(),
      accessModeId: json['accessModeId']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentTurnInputQueueEntryRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentTurnInputQueueEntryRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'content': content,
      'displayText': displayText,
      'contentType': contentType,
      'attachmentNames': attachmentNames?.map((item) => item).toList(),
      'driveRefs': driveRefs?.map((item) => item.toJson()).toList(),
      'turnMode': turnMode,
      'systemPrompt': systemPrompt,
      'runtimeBindingId': runtimeBindingId,
      'requestedModelId': requestedModelId,
      'accessModeId': accessModeId,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentTurnInputQueueReorderEntry {
  final String queueEntryId;
  final String expectedVersion;

  AgentTurnInputQueueReorderEntry({
    required this.queueEntryId,
    required this.expectedVersion
  });

  factory AgentTurnInputQueueReorderEntry.fromJson(Map<String, dynamic> json) {
    return AgentTurnInputQueueReorderEntry(
      queueEntryId: (() {
        final value = json['queueEntryId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueReorderEntry.queueEntryId is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueReorderEntry.expectedVersion is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'queueEntryId': queueEntryId,
      'expectedVersion': expectedVersion,
    };
  }
}

class ReorderAgentTurnInputQueueEntriesRequest {
  final List<AgentTurnInputQueueReorderEntry> orderedEntries;
  final String requestedAt;

  ReorderAgentTurnInputQueueEntriesRequest({
    required this.orderedEntries,
    required this.requestedAt
  });

  factory ReorderAgentTurnInputQueueEntriesRequest.fromJson(Map<String, dynamic> json) {
    return ReorderAgentTurnInputQueueEntriesRequest(
      orderedEntries: (() {
        final list = _sdkworkAsList(json['orderedEntries']);
        if (list == null) {
          throw FormatException('ReorderAgentTurnInputQueueEntriesRequest.orderedEntries is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentTurnInputQueueReorderEntry.fromJson(map);
      })())
            .whereType<AgentTurnInputQueueReorderEntry>()
            .toList();
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ReorderAgentTurnInputQueueEntriesRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'orderedEntries': orderedEntries.map((item) => item.toJson()).toList(),
      'requestedAt': requestedAt,
    };
  }
}

class ClaimNextAgentTurnInputQueueEntryRequest {
  final String claimOwner;
  final int? leaseSeconds;
  final String requestedAt;

  ClaimNextAgentTurnInputQueueEntryRequest({
    required this.claimOwner,
    this.leaseSeconds,
    required this.requestedAt
  });

  factory ClaimNextAgentTurnInputQueueEntryRequest.fromJson(Map<String, dynamic> json) {
    return ClaimNextAgentTurnInputQueueEntryRequest(
      claimOwner: (() {
        final value = json['claimOwner']?.toString();
        if (value == null) {
          throw FormatException('ClaimNextAgentTurnInputQueueEntryRequest.claimOwner is required');
        }
        return value;
      })(),
      leaseSeconds: json['leaseSeconds'] is int ? json['leaseSeconds'] : null,
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ClaimNextAgentTurnInputQueueEntryRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'claimOwner': claimOwner,
      'leaseSeconds': leaseSeconds,
      'requestedAt': requestedAt,
    };
  }
}

class FailAgentTurnInputQueueEntryRequest {
  final String expectedVersion;
  final String fencingToken;
  final String claimToken;
  final String errorCode;
  final String? errorDetail;
  final String requestedAt;

  FailAgentTurnInputQueueEntryRequest({
    required this.expectedVersion,
    required this.fencingToken,
    required this.claimToken,
    required this.errorCode,
    this.errorDetail,
    required this.requestedAt
  });

  factory FailAgentTurnInputQueueEntryRequest.fromJson(Map<String, dynamic> json) {
    return FailAgentTurnInputQueueEntryRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('FailAgentTurnInputQueueEntryRequest.expectedVersion is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('FailAgentTurnInputQueueEntryRequest.fencingToken is required');
        }
        return value;
      })(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('FailAgentTurnInputQueueEntryRequest.claimToken is required');
        }
        return value;
      })(),
      errorCode: (() {
        final value = json['errorCode']?.toString();
        if (value == null) {
          throw FormatException('FailAgentTurnInputQueueEntryRequest.errorCode is required');
        }
        return value;
      })(),
      errorDetail: json['errorDetail']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('FailAgentTurnInputQueueEntryRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'fencingToken': fencingToken,
      'claimToken': claimToken,
      'errorCode': errorCode,
      'errorDetail': errorDetail,
      'requestedAt': requestedAt,
    };
  }
}

class RetryAgentTurnInputQueueEntryRequest {
  final String expectedVersion;
  final String requestedAt;

  RetryAgentTurnInputQueueEntryRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory RetryAgentTurnInputQueueEntryRequest.fromJson(Map<String, dynamic> json) {
    return RetryAgentTurnInputQueueEntryRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('RetryAgentTurnInputQueueEntryRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RetryAgentTurnInputQueueEntryRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentTurnInputQueueEntryResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnInputQueueEntryResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnInputQueueEntryResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnInputQueueEntryResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnInputQueueEntryResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnInputQueueEntryResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntryResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnInputQueueEntryListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnInputQueueEntryListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnInputQueueEntryListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnInputQueueEntryListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnInputQueueEntryListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnInputQueueEntryListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnInputQueueEntryListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ReorderAgentTurnInputQueueEntriesResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ReorderAgentTurnInputQueueEntriesResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ReorderAgentTurnInputQueueEntriesResponse.fromJson(Map<String, dynamic> json) {
    return ReorderAgentTurnInputQueueEntriesResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ReorderAgentTurnInputQueueEntriesResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ReorderAgentTurnInputQueueEntriesResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ReorderAgentTurnInputQueueEntriesResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ClaimNextAgentTurnInputQueueEntryResult {
  final String outcome;
  final AgentTurnInputQueueEntry? entry;
  final String? claimToken;

  ClaimNextAgentTurnInputQueueEntryResult({
    required this.outcome,
    this.entry,
    this.claimToken
  });

  factory ClaimNextAgentTurnInputQueueEntryResult.fromJson(Map<String, dynamic> json) {
    return ClaimNextAgentTurnInputQueueEntryResult(
      outcome: (() {
        final value = json['outcome']?.toString();
        if (value == null) {
          throw FormatException('ClaimNextAgentTurnInputQueueEntryResult.outcome is required');
        }
        return value;
      })(),
      entry: (() {
        final map = _sdkworkAsMap(json['entry']);
        return map == null ? null : AgentTurnInputQueueEntry.fromJson(map);
      })(),
      claimToken: json['claimToken']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'outcome': outcome,
      'entry': entry?.toJson(),
      'claimToken': claimToken,
    };
  }
}

class ClaimNextAgentTurnInputQueueEntryResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ClaimNextAgentTurnInputQueueEntryResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ClaimNextAgentTurnInputQueueEntryResponse.fromJson(Map<String, dynamic> json) {
    return ClaimNextAgentTurnInputQueueEntryResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ClaimNextAgentTurnInputQueueEntryResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ClaimNextAgentTurnInputQueueEntryResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ClearAgentTurnInputQueueEntriesResponse {
  final int code;
  final dynamic data;
  final String traceId;

  ClearAgentTurnInputQueueEntriesResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory ClearAgentTurnInputQueueEntriesResponse.fromJson(Map<String, dynamic> json) {
    return ClearAgentTurnInputQueueEntriesResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ClearAgentTurnInputQueueEntriesResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('ClearAgentTurnInputQueueEntriesResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ClearAgentTurnInputQueueEntriesResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnRecord {
  final String turnId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String agentId;
  final String ownerUserId;
  final String? runtimeBindingId;
  final String? clientRequestId;
  final String idempotencyKey;
  final String payloadHash;
  final String requestItemId;
  final String? responseItemId;
  final String turnMode;
  final String status;
  final String? requestedModelId;
  final String? providerBindingId;
  final String? modelId;
  final String? providerId;
  final String inputTokens;
  final String outputTokens;
  final String cachedTokens;
  final String? finishReason;
  final String? errorCode;
  final String? errorDetail;
  final String? traceId;
  final int attemptCount;
  final int maxAttempts;
  final String? nextRetryAt;
  final String availableAt;
  final String? leaseOwner;
  final String? leaseExpiresAt;
  final String fencingToken;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? startedAt;
  final String? completedAt;
  final String? cancelRequestedAt;
  final String? cancelledAt;
  final String? retentionUntil;

  AgentTurnRecord({
    required this.turnId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    required this.agentId,
    required this.ownerUserId,
    this.runtimeBindingId,
    this.clientRequestId,
    required this.idempotencyKey,
    required this.payloadHash,
    required this.requestItemId,
    this.responseItemId,
    required this.turnMode,
    required this.status,
    this.requestedModelId,
    this.providerBindingId,
    this.modelId,
    this.providerId,
    required this.inputTokens,
    required this.outputTokens,
    required this.cachedTokens,
    this.finishReason,
    this.errorCode,
    this.errorDetail,
    this.traceId,
    required this.attemptCount,
    required this.maxAttempts,
    this.nextRetryAt,
    required this.availableAt,
    this.leaseOwner,
    this.leaseExpiresAt,
    required this.fencingToken,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.startedAt,
    this.completedAt,
    this.cancelRequestedAt,
    this.cancelledAt,
    this.retentionUntil
  });

  factory AgentTurnRecord.fromJson(Map<String, dynamic> json) {
    return AgentTurnRecord(
      turnId: (() {
        final value = json['turnId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.turnId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.sessionId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.agentId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.ownerUserId is required');
        }
        return value;
      })(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      clientRequestId: json['clientRequestId']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.payloadHash is required');
        }
        return value;
      })(),
      requestItemId: (() {
        final value = json['requestItemId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.requestItemId is required');
        }
        return value;
      })(),
      responseItemId: json['responseItemId']?.toString(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.turnMode is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.status is required');
        }
        return value;
      })(),
      requestedModelId: json['requestedModelId']?.toString(),
      providerBindingId: json['providerBindingId']?.toString(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      inputTokens: (() {
        final value = json['inputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.inputTokens is required');
        }
        return value;
      })(),
      outputTokens: (() {
        final value = json['outputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.outputTokens is required');
        }
        return value;
      })(),
      cachedTokens: (() {
        final value = json['cachedTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.cachedTokens is required');
        }
        return value;
      })(),
      finishReason: json['finishReason']?.toString(),
      errorCode: json['errorCode']?.toString(),
      errorDetail: json['errorDetail']?.toString(),
      traceId: json['traceId']?.toString(),
      attemptCount: (() {
        final value = json['attemptCount'];
        if (value is! int) {
          throw FormatException('AgentTurnRecord.attemptCount is required');
        }
        return value;
      })(),
      maxAttempts: (() {
        final value = json['maxAttempts'];
        if (value is! int) {
          throw FormatException('AgentTurnRecord.maxAttempts is required');
        }
        return value;
      })(),
      nextRetryAt: json['nextRetryAt']?.toString(),
      availableAt: (() {
        final value = json['availableAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.availableAt is required');
        }
        return value;
      })(),
      leaseOwner: json['leaseOwner']?.toString(),
      leaseExpiresAt: json['leaseExpiresAt']?.toString(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.fencingToken is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.updatedAt is required');
        }
        return value;
      })(),
      startedAt: json['startedAt']?.toString(),
      completedAt: json['completedAt']?.toString(),
      cancelRequestedAt: json['cancelRequestedAt']?.toString(),
      cancelledAt: json['cancelledAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'turnId': turnId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'agentId': agentId,
      'ownerUserId': ownerUserId,
      'runtimeBindingId': runtimeBindingId,
      'clientRequestId': clientRequestId,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'requestItemId': requestItemId,
      'responseItemId': responseItemId,
      'turnMode': turnMode,
      'status': status,
      'requestedModelId': requestedModelId,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'inputTokens': inputTokens,
      'outputTokens': outputTokens,
      'cachedTokens': cachedTokens,
      'finishReason': finishReason,
      'errorCode': errorCode,
      'errorDetail': errorDetail,
      'traceId': traceId,
      'attemptCount': attemptCount,
      'maxAttempts': maxAttempts,
      'nextRetryAt': nextRetryAt,
      'availableAt': availableAt,
      'leaseOwner': leaseOwner,
      'leaseExpiresAt': leaseExpiresAt,
      'fencingToken': fencingToken,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'startedAt': startedAt,
      'completedAt': completedAt,
      'cancelRequestedAt': cancelRequestedAt,
      'cancelledAt': cancelledAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentTurnResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentTurnRequest {
  final String? turnId;
  final String content;
  final String? contentType;
  final String turnMode;
  final String? systemPrompt;
  final String? runtimeBindingId;
  final String? requestedModelId;
  final String? accessModeId;
  final String idempotencyKey;
  final String payloadHash;
  final String? clientRequestId;
  final List<Map<String, dynamic>>? driveRefs;
  final String requestedAt;

  CreateAgentTurnRequest({
    this.turnId,
    required this.content,
    this.contentType,
    required this.turnMode,
    this.systemPrompt,
    this.runtimeBindingId,
    this.requestedModelId,
    this.accessModeId,
    required this.idempotencyKey,
    required this.payloadHash,
    this.clientRequestId,
    this.driveRefs,
    required this.requestedAt
  });

  factory CreateAgentTurnRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentTurnRequest(
      turnId: json['turnId']?.toString(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.content is required');
        }
        return value;
      })(),
      contentType: json['contentType']?.toString(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.turnMode is required');
        }
        return value;
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      requestedModelId: json['requestedModelId']?.toString(),
      accessModeId: json['accessModeId']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.payloadHash is required');
        }
        return value;
      })(),
      clientRequestId: json['clientRequestId']?.toString(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => _sdkworkAsMap(item))
            .whereType<Map<String, dynamic>>()
            .toList();
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'turnId': turnId,
      'content': content,
      'contentType': contentType,
      'turnMode': turnMode,
      'systemPrompt': systemPrompt,
      'runtimeBindingId': runtimeBindingId,
      'requestedModelId': requestedModelId,
      'accessModeId': accessModeId,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'clientRequestId': clientRequestId,
      'driveRefs': driveRefs?.map((item) => item).toList(),
      'requestedAt': requestedAt,
    };
  }
}

class AgentTurnExecutionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnExecutionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnExecutionResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnExecutionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnExecutionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnExecutionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnExecutionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnRuntimeEvent {
  final String eventId;
  final String type;
  final String version;
  final int sequence;
  final String? occurredAt;
  final String source;
  final String severity;
  final String sessionId;
  final String turnId;
  final String? providerSessionId;
  final String? taskId;
  final String? runId;
  final String? itemId;
  final Map<String, dynamic>? traceContext;
  final String? correlationId;
  final String? causationId;
  final String redactionClassification;
  final String? payloadSchema;
  final Map<String, dynamic> payload;
  final bool replay;

  AgentTurnRuntimeEvent({
    required this.eventId,
    required this.type,
    required this.version,
    required this.sequence,
    required this.occurredAt,
    required this.source,
    required this.severity,
    required this.sessionId,
    required this.turnId,
    required this.providerSessionId,
    required this.taskId,
    required this.runId,
    required this.itemId,
    required this.traceContext,
    required this.correlationId,
    required this.causationId,
    required this.redactionClassification,
    required this.payloadSchema,
    required this.payload,
    required this.replay
  });

  factory AgentTurnRuntimeEvent.fromJson(Map<String, dynamic> json) {
    return AgentTurnRuntimeEvent(
      eventId: (() {
        final value = json['eventId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.eventId is required');
        }
        return value;
      })(),
      type: (() {
        final value = json['type']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.type is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.version is required');
        }
        return value;
      })(),
      sequence: (() {
        final value = json['sequence'];
        if (value is! int) {
          throw FormatException('AgentTurnRuntimeEvent.sequence is required');
        }
        return value;
      })(),
      occurredAt: (() {
        if (!json.containsKey('occurredAt')) {
          throw FormatException('AgentTurnRuntimeEvent.occurredAt is required');
        }
        final _sdkworkRequiredValue = json['occurredAt'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.occurredAt is required');
        }
        return value;
      })();
      })(),
      source: (() {
        final value = json['source']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.source is required');
        }
        return value;
      })(),
      severity: (() {
        final value = json['severity']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.severity is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.sessionId is required');
        }
        return value;
      })(),
      turnId: (() {
        final value = json['turnId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.turnId is required');
        }
        return value;
      })(),
      providerSessionId: (() {
        if (!json.containsKey('providerSessionId')) {
          throw FormatException('AgentTurnRuntimeEvent.providerSessionId is required');
        }
        final _sdkworkRequiredValue = json['providerSessionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.providerSessionId is required');
        }
        return value;
      })();
      })(),
      taskId: (() {
        if (!json.containsKey('taskId')) {
          throw FormatException('AgentTurnRuntimeEvent.taskId is required');
        }
        final _sdkworkRequiredValue = json['taskId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.taskId is required');
        }
        return value;
      })();
      })(),
      runId: (() {
        if (!json.containsKey('runId')) {
          throw FormatException('AgentTurnRuntimeEvent.runId is required');
        }
        final _sdkworkRequiredValue = json['runId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.runId is required');
        }
        return value;
      })();
      })(),
      itemId: (() {
        if (!json.containsKey('itemId')) {
          throw FormatException('AgentTurnRuntimeEvent.itemId is required');
        }
        final _sdkworkRequiredValue = json['itemId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.itemId is required');
        }
        return value;
      })();
      })(),
      traceContext: (() {
        if (!json.containsKey('traceContext')) {
          throw FormatException('AgentTurnRuntimeEvent.traceContext is required');
        }
        final _sdkworkRequiredValue = json['traceContext'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final map = _sdkworkAsMap(_sdkworkRequiredValue);
        if (map == null) {
          throw FormatException('AgentTurnRuntimeEvent.traceContext is required');
        }
        return map;
      })();
      })(),
      correlationId: (() {
        if (!json.containsKey('correlationId')) {
          throw FormatException('AgentTurnRuntimeEvent.correlationId is required');
        }
        final _sdkworkRequiredValue = json['correlationId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.correlationId is required');
        }
        return value;
      })();
      })(),
      causationId: (() {
        if (!json.containsKey('causationId')) {
          throw FormatException('AgentTurnRuntimeEvent.causationId is required');
        }
        final _sdkworkRequiredValue = json['causationId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.causationId is required');
        }
        return value;
      })();
      })(),
      redactionClassification: (() {
        final value = json['redactionClassification']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.redactionClassification is required');
        }
        return value;
      })(),
      payloadSchema: (() {
        if (!json.containsKey('payloadSchema')) {
          throw FormatException('AgentTurnRuntimeEvent.payloadSchema is required');
        }
        final _sdkworkRequiredValue = json['payloadSchema'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRuntimeEvent.payloadSchema is required');
        }
        return value;
      })();
      })(),
      payload: (() {
        final map = _sdkworkAsMap(json['payload']);
        if (map == null) {
          throw FormatException('AgentTurnRuntimeEvent.payload is required');
        }
        return map;
      })(),
      replay: (() {
        final value = json['replay'];
        if (value is! bool) {
          throw FormatException('AgentTurnRuntimeEvent.replay is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'eventId': eventId,
      'type': type,
      'version': version,
      'sequence': sequence,
      'occurredAt': occurredAt,
      'source': source,
      'severity': severity,
      'sessionId': sessionId,
      'turnId': turnId,
      'providerSessionId': providerSessionId,
      'taskId': taskId,
      'runId': runId,
      'itemId': itemId,
      'traceContext': traceContext,
      'correlationId': correlationId,
      'causationId': causationId,
      'redactionClassification': redactionClassification,
      'payloadSchema': payloadSchema,
      'payload': payload,
      'replay': replay,
    };
  }
}

class AgentTurnStreamEvent {
  final String eventType;
  final AgentTurnRuntimeEvent? event;
  final int? index;
  final String? delta;
  final AgentTurnExecutionResponse? response;

  AgentTurnStreamEvent({
    required this.eventType,
    this.event,
    this.index,
    this.delta,
    this.response
  });

  factory AgentTurnStreamEvent.fromJson(Map<String, dynamic> json) {
    return AgentTurnStreamEvent(
      eventType: (() {
        final value = json['eventType']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnStreamEvent.eventType is required');
        }
        return value;
      })(),
      event: (() {
        final map = _sdkworkAsMap(json['event']);
        return map == null ? null : AgentTurnRuntimeEvent.fromJson(map);
      })(),
      index: json['index'] is int ? json['index'] : null,
      delta: json['delta']?.toString(),
      response: (() {
        final map = _sdkworkAsMap(json['response']);
        return map == null ? null : AgentTurnExecutionResponse.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'eventType': eventType,
      'event': event?.toJson(),
      'index': index,
      'delta': delta,
      'response': response?.toJson(),
    };
  }
}

class CancelAgentTurnRequest {
  final String expectedVersion;
  final String requestedAt;

  CancelAgentTurnRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory CancelAgentTurnRequest.fromJson(Map<String, dynamic> json) {
    return CancelAgentTurnRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTurnRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTurnRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentInteractionOption {
  final String value;
  final String label;

  AgentInteractionOption({
    required this.value,
    required this.label
  });

  factory AgentInteractionOption.fromJson(Map<String, dynamic> json) {
    return AgentInteractionOption(
      value: (() {
        final value = json['value']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionOption.value is required');
        }
        return value;
      })(),
      label: (() {
        final value = json['label']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionOption.label is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'value': value,
      'label': label,
    };
  }
}

class LegacyAgentInteractionResolution implements AgentInteractionResolution {
  final String outcome;
  final String? answer;
  final String? selectedOptionValue;
  final String? reason;

  LegacyAgentInteractionResolution({
    required this.outcome,
    this.answer,
    this.selectedOptionValue,
    this.reason
  });

  factory LegacyAgentInteractionResolution.fromJson(Map<String, dynamic> json) {
    return LegacyAgentInteractionResolution(
      outcome: (() {
        final value = json['outcome']?.toString();
        if (value == null) {
          throw FormatException('LegacyAgentInteractionResolution.outcome is required');
        }
        return value;
      })(),
      answer: json['answer']?.toString(),
      selectedOptionValue: json['selectedOptionValue']?.toString(),
      reason: json['reason']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'outcome': outcome,
      'answer': answer,
      'selectedOptionValue': selectedOptionValue,
      'reason': reason,
    };
  }
}

class AgentInteractionQuestionOption {
  final String label;
  final String? description;

  AgentInteractionQuestionOption({
    required this.label,
    this.description
  });

  factory AgentInteractionQuestionOption.fromJson(Map<String, dynamic> json) {
    return AgentInteractionQuestionOption(
      label: (() {
        final value = json['label']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionQuestionOption.label is required');
        }
        return value;
      })(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'label': label,
      'description': description,
    };
  }
}

class AgentInteractionQuestion {
  final String id;
  final String header;
  final String prompt;
  final bool allowOther;
  final bool secret;
  final List<AgentInteractionQuestionOption>? options;

  AgentInteractionQuestion({
    required this.id,
    required this.header,
    required this.prompt,
    required this.allowOther,
    required this.secret,
    required this.options
  });

  factory AgentInteractionQuestion.fromJson(Map<String, dynamic> json) {
    return AgentInteractionQuestion(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionQuestion.id is required');
        }
        return value;
      })(),
      header: (() {
        final value = json['header']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionQuestion.header is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionQuestion.prompt is required');
        }
        return value;
      })(),
      allowOther: (() {
        final value = json['allowOther'];
        if (value is! bool) {
          throw FormatException('AgentInteractionQuestion.allowOther is required');
        }
        return value;
      })(),
      secret: (() {
        final value = json['secret'];
        if (value is! bool) {
          throw FormatException('AgentInteractionQuestion.secret is required');
        }
        return value;
      })(),
      options: (() {
        if (!json.containsKey('options')) {
          throw FormatException('AgentInteractionQuestion.options is required');
        }
        final _sdkworkRequiredValue = json['options'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final list = _sdkworkAsList(_sdkworkRequiredValue);
        if (list == null) {
          throw FormatException('AgentInteractionQuestion.options is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionQuestionOption.fromJson(map);
      })())
            .whereType<AgentInteractionQuestionOption>()
            .toList();
      })();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'header': header,
      'prompt': prompt,
      'allowOther': allowOther,
      'secret': secret,
      'options': options?.map((item) => item.toJson()).toList(),
    };
  }
}

class AgentInteractionRequestData {
  final String? itemId;
  final String? command;
  final String? cwd;
  final String? reason;
  final Map<String, dynamic>? changes;
  final Map<String, dynamic>? proposedExecPolicyAmendment;
  final Map<String, dynamic>? proposedNetworkPolicyAmendment;
  final List<AgentInteractionQuestion>? questions;
  final int? autoResolutionMs;
  final bool? isBlocking;
  final String? question;
  final List<AgentInteractionQuestionOption>? options;
  final bool? allowMultiple;
  final String? submitLabel;
  final String? skipLabel;
  final String? step;
  final String? mode;
  final String? serverName;
  final String? message;
  final String? elicitationId;
  final String? url;
  final dynamic requestedSchema;
  final Map<String, dynamic>? requestedPermissions;

  AgentInteractionRequestData({
    this.itemId,
    this.command,
    this.cwd,
    this.reason,
    this.changes,
    this.proposedExecPolicyAmendment,
    this.proposedNetworkPolicyAmendment,
    this.questions,
    this.autoResolutionMs,
    this.isBlocking,
    this.question,
    this.options,
    this.allowMultiple,
    this.submitLabel,
    this.skipLabel,
    this.step,
    this.mode,
    this.serverName,
    this.message,
    this.elicitationId,
    this.url,
    this.requestedSchema,
    this.requestedPermissions
  });

  factory AgentInteractionRequestData.fromJson(Map<String, dynamic> json) {
    return AgentInteractionRequestData(
      itemId: json['itemId']?.toString(),
      command: json['command']?.toString(),
      cwd: json['cwd']?.toString(),
      reason: json['reason']?.toString(),
      changes: _sdkworkAsMap(json['changes']),
      proposedExecPolicyAmendment: _sdkworkAsMap(json['proposedExecPolicyAmendment']),
      proposedNetworkPolicyAmendment: _sdkworkAsMap(json['proposedNetworkPolicyAmendment']),
      questions: (() {
        final list = _sdkworkAsList(json['questions']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionQuestion.fromJson(map);
      })())
            .whereType<AgentInteractionQuestion>()
            .toList();
      })(),
      autoResolutionMs: json['autoResolutionMs'] is int ? json['autoResolutionMs'] : null,
      isBlocking: json['isBlocking'] is bool ? json['isBlocking'] : null,
      question: json['question']?.toString(),
      options: (() {
        final list = _sdkworkAsList(json['options']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionQuestionOption.fromJson(map);
      })())
            .whereType<AgentInteractionQuestionOption>()
            .toList();
      })(),
      allowMultiple: json['allowMultiple'] is bool ? json['allowMultiple'] : null,
      submitLabel: json['submitLabel']?.toString(),
      skipLabel: json['skipLabel']?.toString(),
      step: json['step']?.toString(),
      mode: json['mode']?.toString(),
      serverName: json['serverName']?.toString(),
      message: json['message']?.toString(),
      elicitationId: json['elicitationId']?.toString(),
      url: json['url']?.toString(),
      requestedSchema: json['requestedSchema'],
      requestedPermissions: _sdkworkAsMap(json['requestedPermissions'])
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'itemId': itemId,
      'command': command,
      'cwd': cwd,
      'reason': reason,
      'changes': changes,
      'proposedExecPolicyAmendment': proposedExecPolicyAmendment,
      'proposedNetworkPolicyAmendment': proposedNetworkPolicyAmendment,
      'questions': questions?.map((item) => item.toJson()).toList(),
      'autoResolutionMs': autoResolutionMs,
      'isBlocking': isBlocking,
      'question': question,
      'options': options?.map((item) => item.toJson()).toList(),
      'allowMultiple': allowMultiple,
      'submitLabel': submitLabel,
      'skipLabel': skipLabel,
      'step': step,
      'mode': mode,
      'serverName': serverName,
      'message': message,
      'elicitationId': elicitationId,
      'url': url,
      'requestedSchema': requestedSchema,
      'requestedPermissions': requestedPermissions,
    };
  }
}

class AgentInteractionCorrelation {
  final String modelRequestId;
  final String providerId;
  final String? providerInteractionId;
  final String? providerItemId;
  final dynamic providerRequestId;
  final String providerRequestIdType;
  final String providerSessionId;
  final String? providerToolCallId;
  final String? providerToolName;
  final String? providerToolNamespace;
  final String providerTurnId;
  final String protocolMethod;

  AgentInteractionCorrelation({
    required this.modelRequestId,
    required this.providerId,
    required this.providerInteractionId,
    required this.providerItemId,
    required this.providerRequestId,
    required this.providerRequestIdType,
    required this.providerSessionId,
    required this.providerToolCallId,
    required this.providerToolName,
    required this.providerToolNamespace,
    required this.providerTurnId,
    required this.protocolMethod
  });

  factory AgentInteractionCorrelation.fromJson(Map<String, dynamic> json) {
    return AgentInteractionCorrelation(
      modelRequestId: (() {
        final value = json['modelRequestId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.modelRequestId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerId is required');
        }
        return value;
      })(),
      providerInteractionId: (() {
        if (!json.containsKey('providerInteractionId')) {
          throw FormatException('AgentInteractionCorrelation.providerInteractionId is required');
        }
        final _sdkworkRequiredValue = json['providerInteractionId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerInteractionId is required');
        }
        return value;
      })();
      })(),
      providerItemId: (() {
        if (!json.containsKey('providerItemId')) {
          throw FormatException('AgentInteractionCorrelation.providerItemId is required');
        }
        final _sdkworkRequiredValue = json['providerItemId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerItemId is required');
        }
        return value;
      })();
      })(),
      providerRequestId: (() {
        final value = json['providerRequestId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerRequestId is required');
        }
        return value;
      })(),
      providerRequestIdType: (() {
        final value = json['providerRequestIdType']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerRequestIdType is required');
        }
        return value;
      })(),
      providerSessionId: (() {
        final value = json['providerSessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerSessionId is required');
        }
        return value;
      })(),
      providerToolCallId: (() {
        if (!json.containsKey('providerToolCallId')) {
          throw FormatException('AgentInteractionCorrelation.providerToolCallId is required');
        }
        final _sdkworkRequiredValue = json['providerToolCallId'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerToolCallId is required');
        }
        return value;
      })();
      })(),
      providerToolName: (() {
        if (!json.containsKey('providerToolName')) {
          throw FormatException('AgentInteractionCorrelation.providerToolName is required');
        }
        final _sdkworkRequiredValue = json['providerToolName'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerToolName is required');
        }
        return value;
      })();
      })(),
      providerToolNamespace: (() {
        if (!json.containsKey('providerToolNamespace')) {
          throw FormatException('AgentInteractionCorrelation.providerToolNamespace is required');
        }
        final _sdkworkRequiredValue = json['providerToolNamespace'];
        if (_sdkworkRequiredValue == null) {
          return null;
        }
        return (() {
        final value = _sdkworkRequiredValue?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerToolNamespace is required');
        }
        return value;
      })();
      })(),
      providerTurnId: (() {
        final value = json['providerTurnId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.providerTurnId is required');
        }
        return value;
      })(),
      protocolMethod: (() {
        final value = json['protocolMethod']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionCorrelation.protocolMethod is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'modelRequestId': modelRequestId,
      'providerId': providerId,
      'providerInteractionId': providerInteractionId,
      'providerItemId': providerItemId,
      'providerRequestId': providerRequestId,
      'providerRequestIdType': providerRequestIdType,
      'providerSessionId': providerSessionId,
      'providerToolCallId': providerToolCallId,
      'providerToolName': providerToolName,
      'providerToolNamespace': providerToolNamespace,
      'providerTurnId': providerTurnId,
      'protocolMethod': protocolMethod,
    };
  }
}

class AgentInteractionRequest {
  final int schemaVersion;
  final String category;
  final String kind;
  final List<String> allowedActions;
  final AgentInteractionRequestData data;
  final AgentInteractionCorrelation? correlation;

  AgentInteractionRequest({
    required this.schemaVersion,
    required this.category,
    required this.kind,
    required this.allowedActions,
    required this.data,
    this.correlation
  });

  factory AgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return AgentInteractionRequest(
      schemaVersion: (() {
        final value = json['schemaVersion'];
        if (value is! int) {
          throw FormatException('AgentInteractionRequest.schemaVersion is required');
        }
        return value;
      })(),
      category: (() {
        final value = json['category']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRequest.category is required');
        }
        return value;
      })(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRequest.kind is required');
        }
        return value;
      })(),
      allowedActions: (() {
        final list = _sdkworkAsList(json['allowedActions']);
        if (list == null) {
          throw FormatException('AgentInteractionRequest.allowedActions is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionRequest.data is required');
        }
        return AgentInteractionRequestData.fromJson(map);
      })(),
      correlation: (() {
        final map = _sdkworkAsMap(json['correlation']);
        return map == null ? null : AgentInteractionCorrelation.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'schemaVersion': schemaVersion,
      'category': category,
      'kind': kind,
      'allowedActions': allowedActions.map((item) => item).toList(),
      'data': data.toJson(),
      'correlation': correlation?.toJson(),
    };
  }
}

class TypedAgentInteractionResolution implements AgentInteractionResolution {
  final String action;
  final Map<String, List<String>>? answers;
  final List<String>? selectedOptions;
  final String? freeformAnswer;
  final List<String>? selectedSources;
  final List<String>? selectedRoles;
  final Map<String, dynamic>? execPolicyAmendment;
  final Map<String, dynamic>? networkPolicyAmendment;
  final Map<String, dynamic>? permissions;
  final String? scope;
  final bool? strictAutoReview;
  final dynamic content;
  final Map<String, dynamic>? metadata;

  TypedAgentInteractionResolution({
    required this.action,
    this.answers,
    this.selectedOptions,
    this.freeformAnswer,
    this.selectedSources,
    this.selectedRoles,
    this.execPolicyAmendment,
    this.networkPolicyAmendment,
    this.permissions,
    this.scope,
    this.strictAutoReview,
    this.content,
    this.metadata
  });

  factory TypedAgentInteractionResolution.fromJson(Map<String, dynamic> json) {
    return TypedAgentInteractionResolution(
      action: (() {
        final value = json['action']?.toString();
        if (value == null) {
          throw FormatException('TypedAgentInteractionResolution.action is required');
        }
        return value;
      })(),
      answers: (() {
        final map = _sdkworkAsMap(json['answers']);
        if (map == null) {
          return null;
        }
        final result = <String, List<String>>{};
        map.forEach((key, item) {
          final deserialized = (() {
        final list = _sdkworkAsList(item);
        if (list == null) {
          return null;
        }
        return list
            .map((nestedItem) => nestedItem?.toString())
            .whereType<String>()
            .toList();
      })();
          if (deserialized is List<String>) {
            result[key] = deserialized;
          }
        });
        return result;
      })(),
      selectedOptions: (() {
        final list = _sdkworkAsList(json['selectedOptions']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      freeformAnswer: json['freeformAnswer']?.toString(),
      selectedSources: (() {
        final list = _sdkworkAsList(json['selectedSources']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      selectedRoles: (() {
        final list = _sdkworkAsList(json['selectedRoles']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      execPolicyAmendment: _sdkworkAsMap(json['execPolicyAmendment']),
      networkPolicyAmendment: _sdkworkAsMap(json['networkPolicyAmendment']),
      permissions: _sdkworkAsMap(json['permissions']),
      scope: json['scope']?.toString(),
      strictAutoReview: json['strictAutoReview'] is bool ? json['strictAutoReview'] : null,
      content: json['content'],
      metadata: _sdkworkAsMap(json['metadata'])
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'action': action,
      'answers': answers?.map((key, item) => MapEntry(key, item.map((nestedItem) => nestedItem).toList())),
      'selectedOptions': selectedOptions?.map((item) => item).toList(),
      'freeformAnswer': freeformAnswer,
      'selectedSources': selectedSources?.map((item) => item).toList(),
      'selectedRoles': selectedRoles?.map((item) => item).toList(),
      'execPolicyAmendment': execPolicyAmendment,
      'networkPolicyAmendment': networkPolicyAmendment,
      'permissions': permissions,
      'scope': scope,
      'strictAutoReview': strictAutoReview,
      'content': content,
      'metadata': metadata,
    };
  }
}

class AgentInteractionResolution {


  AgentInteractionResolution();

  factory AgentInteractionResolution.fromJson(Map<String, dynamic> json) {
    return AgentInteractionResolution();
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{};
  }
}

class AgentInteractionRecord {
  final String interactionId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String? providerInteractionId;
  final String kind;
  final String status;
  final String prompt;
  final List<AgentInteractionOption> options;
  final AgentInteractionRequest? request;
  final AgentInteractionResolution? resolution;
  final String? claimOwner;
  final String? claimExpiresAt;
  final String fencingToken;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? resolvedAt;
  final String? retentionUntil;

  AgentInteractionRecord({
    required this.interactionId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.turnId,
    this.runtimeBindingId,
    this.providerInteractionId,
    required this.kind,
    required this.status,
    required this.prompt,
    required this.options,
    this.request,
    this.resolution,
    this.claimOwner,
    this.claimExpiresAt,
    required this.fencingToken,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.resolvedAt,
    this.retentionUntil
  });

  factory AgentInteractionRecord.fromJson(Map<String, dynamic> json) {
    return AgentInteractionRecord(
      interactionId: (() {
        final value = json['interactionId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.interactionId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.sessionId is required');
        }
        return value;
      })(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      providerInteractionId: json['providerInteractionId']?.toString(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.kind is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.status is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.prompt is required');
        }
        return value;
      })(),
      options: (() {
        final list = _sdkworkAsList(json['options']);
        if (list == null) {
          throw FormatException('AgentInteractionRecord.options is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionOption.fromJson(map);
      })())
            .whereType<AgentInteractionOption>()
            .toList();
      })(),
      request: (() {
        final map = _sdkworkAsMap(json['request']);
        return map == null ? null : AgentInteractionRequest.fromJson(map);
      })(),
      resolution: (() {
        final map = _sdkworkAsMap(json['resolution']);
        return map == null ? null : AgentInteractionResolution.fromJson(map);
      })(),
      claimOwner: json['claimOwner']?.toString(),
      claimExpiresAt: json['claimExpiresAt']?.toString(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.fencingToken is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.updatedAt is required');
        }
        return value;
      })(),
      resolvedAt: json['resolvedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'interactionId': interactionId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'providerInteractionId': providerInteractionId,
      'kind': kind,
      'status': status,
      'prompt': prompt,
      'options': options.map((item) => item.toJson()).toList(),
      'request': request?.toJson(),
      'resolution': resolution?.toJson(),
      'claimOwner': claimOwner,
      'claimExpiresAt': claimExpiresAt,
      'fencingToken': fencingToken,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'resolvedAt': resolvedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentInteractionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentInteractionListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionListResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentInteractionRequest {
  final String? interactionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String? providerInteractionId;
  final String kind;
  final String prompt;
  final List<AgentInteractionOption>? options;
  final AgentInteractionRequest? request;
  final String? retentionUntil;
  final String requestedAt;

  CreateAgentInteractionRequest({
    this.interactionId,
    this.turnId,
    this.runtimeBindingId,
    this.providerInteractionId,
    required this.kind,
    required this.prompt,
    this.options,
    this.request,
    this.retentionUntil,
    required this.requestedAt
  });

  factory CreateAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentInteractionRequest(
      interactionId: json['interactionId']?.toString(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      providerInteractionId: json['providerInteractionId']?.toString(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.kind is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.prompt is required');
        }
        return value;
      })(),
      options: (() {
        final list = _sdkworkAsList(json['options']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionOption.fromJson(map);
      })())
            .whereType<AgentInteractionOption>()
            .toList();
      })(),
      request: (() {
        final map = _sdkworkAsMap(json['request']);
        return map == null ? null : AgentInteractionRequest.fromJson(map);
      })(),
      retentionUntil: json['retentionUntil']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'interactionId': interactionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'providerInteractionId': providerInteractionId,
      'kind': kind,
      'prompt': prompt,
      'options': options?.map((item) => item.toJson()).toList(),
      'request': request?.toJson(),
      'retentionUntil': retentionUntil,
      'requestedAt': requestedAt,
    };
  }
}

class ClaimAgentInteractionRequest {
  final String claimOwner;
  final int? leaseSeconds;
  final String expectedVersion;
  final String requestedAt;

  ClaimAgentInteractionRequest({
    required this.claimOwner,
    this.leaseSeconds,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ClaimAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return ClaimAgentInteractionRequest(
      claimOwner: (() {
        final value = json['claimOwner']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.claimOwner is required');
        }
        return value;
      })(),
      leaseSeconds: json['leaseSeconds'] is int ? json['leaseSeconds'] : null,
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'claimOwner': claimOwner,
      'leaseSeconds': leaseSeconds,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentInteractionClaimResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionClaimResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionClaimResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionClaimResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionClaimResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionClaimResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionClaimResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ApproveAgentInteractionRequest {
  final bool approved;
  final String? reason;
  final String claimToken;
  final String fencingToken;
  final String expectedVersion;
  final String requestedAt;

  ApproveAgentInteractionRequest({
    required this.approved,
    this.reason,
    required this.claimToken,
    required this.fencingToken,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ApproveAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return ApproveAgentInteractionRequest(
      approved: (() {
        final value = json['approved'];
        if (value is! bool) {
          throw FormatException('ApproveAgentInteractionRequest.approved is required');
        }
        return value;
      })(),
      reason: json['reason']?.toString(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.claimToken is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.fencingToken is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'approved': approved,
      'reason': reason,
      'claimToken': claimToken,
      'fencingToken': fencingToken,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AnswerAgentInteractionRequest {
  final String answer;
  final String? selectedOptionValue;
  final bool rejected;
  final String claimToken;
  final String fencingToken;
  final String expectedVersion;
  final String requestedAt;

  AnswerAgentInteractionRequest({
    required this.answer,
    this.selectedOptionValue,
    required this.rejected,
    required this.claimToken,
    required this.fencingToken,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory AnswerAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return AnswerAgentInteractionRequest(
      answer: (() {
        final value = json['answer']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.answer is required');
        }
        return value;
      })(),
      selectedOptionValue: json['selectedOptionValue']?.toString(),
      rejected: (() {
        final value = json['rejected'];
        if (value is! bool) {
          throw FormatException('AnswerAgentInteractionRequest.rejected is required');
        }
        return value;
      })(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.claimToken is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.fencingToken is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'answer': answer,
      'selectedOptionValue': selectedOptionValue,
      'rejected': rejected,
      'claimToken': claimToken,
      'fencingToken': fencingToken,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class ResolveAgentInteractionRequest {
  final TypedAgentInteractionResolution resolution;
  final String claimToken;
  final String fencingToken;
  final String expectedVersion;
  final String requestedAt;

  ResolveAgentInteractionRequest({
    required this.resolution,
    required this.claimToken,
    required this.fencingToken,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ResolveAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return ResolveAgentInteractionRequest(
      resolution: (() {
        final map = _sdkworkAsMap(json['resolution']);
        if (map == null) {
          throw FormatException('ResolveAgentInteractionRequest.resolution is required');
        }
        return TypedAgentInteractionResolution.fromJson(map);
      })(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('ResolveAgentInteractionRequest.claimToken is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('ResolveAgentInteractionRequest.fencingToken is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ResolveAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ResolveAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'resolution': resolution.toJson(),
      'claimToken': claimToken,
      'fencingToken': fencingToken,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentSessionCheckpointRecord {
  final String checkpointId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String checkpointKind;
  final String? providerCheckpointRef;
  final String? driveSpaceId;
  final String? driveNodeId;
  final bool resumable;
  final String status;
  final String createdBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? restoredAt;
  final String? invalidatedAt;
  final String? retentionUntil;

  AgentSessionCheckpointRecord({
    required this.checkpointId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.turnId,
    this.runtimeBindingId,
    required this.checkpointKind,
    this.providerCheckpointRef,
    this.driveSpaceId,
    this.driveNodeId,
    required this.resumable,
    required this.status,
    required this.createdBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.restoredAt,
    this.invalidatedAt,
    this.retentionUntil
  });

  factory AgentSessionCheckpointRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointRecord(
      checkpointId: (() {
        final value = json['checkpointId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.checkpointId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.sessionId is required');
        }
        return value;
      })(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      checkpointKind: (() {
        final value = json['checkpointKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.checkpointKind is required');
        }
        return value;
      })(),
      providerCheckpointRef: json['providerCheckpointRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveNodeId: json['driveNodeId']?.toString(),
      resumable: (() {
        final value = json['resumable'];
        if (value is! bool) {
          throw FormatException('AgentSessionCheckpointRecord.resumable is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.status is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.createdBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.updatedAt is required');
        }
        return value;
      })(),
      restoredAt: json['restoredAt']?.toString(),
      invalidatedAt: json['invalidatedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'checkpointId': checkpointId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'checkpointKind': checkpointKind,
      'providerCheckpointRef': providerCheckpointRef,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'resumable': resumable,
      'status': status,
      'createdBy': createdBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'restoredAt': restoredAt,
      'invalidatedAt': invalidatedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionCheckpointResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionCheckpointResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionCheckpointResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionCheckpointResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionCheckpointResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionCheckpointListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionCheckpointListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionCheckpointListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionCheckpointListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionCheckpointListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionCheckpointRequest {
  final String? checkpointId;
  final String? turnId;
  final String? runtimeBindingId;
  final String checkpointKind;
  final String? providerCheckpointRef;
  final String? driveSpaceId;
  final String? driveNodeId;
  final bool resumable;
  final String? retentionUntil;
  final String requestedAt;

  CreateAgentSessionCheckpointRequest({
    this.checkpointId,
    this.turnId,
    this.runtimeBindingId,
    required this.checkpointKind,
    this.providerCheckpointRef,
    this.driveSpaceId,
    this.driveNodeId,
    required this.resumable,
    this.retentionUntil,
    required this.requestedAt
  });

  factory CreateAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionCheckpointRequest(
      checkpointId: json['checkpointId']?.toString(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      checkpointKind: (() {
        final value = json['checkpointKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionCheckpointRequest.checkpointKind is required');
        }
        return value;
      })(),
      providerCheckpointRef: json['providerCheckpointRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveNodeId: json['driveNodeId']?.toString(),
      resumable: (() {
        final value = json['resumable'];
        if (value is! bool) {
          throw FormatException('CreateAgentSessionCheckpointRequest.resumable is required');
        }
        return value;
      })(),
      retentionUntil: json['retentionUntil']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'checkpointId': checkpointId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'checkpointKind': checkpointKind,
      'providerCheckpointRef': providerCheckpointRef,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'resumable': resumable,
      'retentionUntil': retentionUntil,
      'requestedAt': requestedAt,
    };
  }
}

class RestoreAgentSessionCheckpointRequest {
  final String expectedVersion;
  final String requestedAt;

  RestoreAgentSessionCheckpointRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory RestoreAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return RestoreAgentSessionCheckpointRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentSessionCheckpointRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class InvalidateAgentSessionCheckpointRequest {
  final String? reason;
  final String expectedVersion;
  final String requestedAt;

  InvalidateAgentSessionCheckpointRequest({
    this.reason,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory InvalidateAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return InvalidateAgentSessionCheckpointRequest(
      reason: json['reason']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('InvalidateAgentSessionCheckpointRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('InvalidateAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'reason': reason,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentSessionRuntimeBindingRecord {
  final String runtimeBindingId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? runtimeLocationId;
  final String hostMode;
  final String transportKind;
  final String providerBindingId;
  final String modelId;
  final String providerId;
  final String? providerSessionId;
  final String? providerSessionTreeId;
  final String? providerParentSessionId;
  final String? providerForkedFromSessionId;
  final String? providerTitle;
  final String? providerTitleSource;
  final String? providerPreview;
  final String? providerCreatedAt;
  final String? providerUpdatedAt;
  final String? providerRecencyAt;
  final bool? providerPinned;
  final bool? providerArchived;
  final bool? providerVisible;
  final String? providerSortKey;
  final String? providerSource;
  final String status;
  final bool isCurrent;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? activatedAt;
  final String? deactivatedAt;

  AgentSessionRuntimeBindingRecord({
    required this.runtimeBindingId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.runtimeLocationId,
    required this.hostMode,
    required this.transportKind,
    required this.providerBindingId,
    required this.modelId,
    required this.providerId,
    this.providerSessionId,
    this.providerSessionTreeId,
    this.providerParentSessionId,
    this.providerForkedFromSessionId,
    this.providerTitle,
    this.providerTitleSource,
    this.providerPreview,
    this.providerCreatedAt,
    this.providerUpdatedAt,
    this.providerRecencyAt,
    this.providerPinned,
    this.providerArchived,
    this.providerVisible,
    this.providerSortKey,
    this.providerSource,
    required this.status,
    required this.isCurrent,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.activatedAt,
    this.deactivatedAt
  });

  factory AgentSessionRuntimeBindingRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingRecord(
      runtimeBindingId: (() {
        final value = json['runtimeBindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.runtimeBindingId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.sessionId is required');
        }
        return value;
      })(),
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      hostMode: (() {
        final value = json['hostMode']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.hostMode is required');
        }
        return value;
      })(),
      transportKind: (() {
        final value = json['transportKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.transportKind is required');
        }
        return value;
      })(),
      providerBindingId: (() {
        final value = json['providerBindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.providerBindingId is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.modelId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.providerId is required');
        }
        return value;
      })(),
      providerSessionId: json['providerSessionId']?.toString(),
      providerSessionTreeId: json['providerSessionTreeId']?.toString(),
      providerParentSessionId: json['providerParentSessionId']?.toString(),
      providerForkedFromSessionId: json['providerForkedFromSessionId']?.toString(),
      providerTitle: json['providerTitle']?.toString(),
      providerTitleSource: json['providerTitleSource']?.toString(),
      providerPreview: json['providerPreview']?.toString(),
      providerCreatedAt: json['providerCreatedAt']?.toString(),
      providerUpdatedAt: json['providerUpdatedAt']?.toString(),
      providerRecencyAt: json['providerRecencyAt']?.toString(),
      providerPinned: json['providerPinned'] is bool ? json['providerPinned'] : null,
      providerArchived: json['providerArchived'] is bool ? json['providerArchived'] : null,
      providerVisible: json['providerVisible'] is bool ? json['providerVisible'] : null,
      providerSortKey: json['providerSortKey']?.toString(),
      providerSource: json['providerSource']?.toString(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.status is required');
        }
        return value;
      })(),
      isCurrent: (() {
        final value = json['isCurrent'];
        if (value is! bool) {
          throw FormatException('AgentSessionRuntimeBindingRecord.isCurrent is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.updatedAt is required');
        }
        return value;
      })(),
      activatedAt: json['activatedAt']?.toString(),
      deactivatedAt: json['deactivatedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeBindingId': runtimeBindingId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'runtimeLocationId': runtimeLocationId,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'providerSessionId': providerSessionId,
      'providerSessionTreeId': providerSessionTreeId,
      'providerParentSessionId': providerParentSessionId,
      'providerForkedFromSessionId': providerForkedFromSessionId,
      'providerTitle': providerTitle,
      'providerTitleSource': providerTitleSource,
      'providerPreview': providerPreview,
      'providerCreatedAt': providerCreatedAt,
      'providerUpdatedAt': providerUpdatedAt,
      'providerRecencyAt': providerRecencyAt,
      'providerPinned': providerPinned,
      'providerArchived': providerArchived,
      'providerVisible': providerVisible,
      'providerSortKey': providerSortKey,
      'providerSource': providerSource,
      'status': status,
      'isCurrent': isCurrent,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'activatedAt': activatedAt,
      'deactivatedAt': deactivatedAt,
    };
  }
}

class AgentSessionRuntimeBindingResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionRuntimeBindingResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionRuntimeBindingResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionRuntimeBindingResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionRuntimeBindingResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionRuntimeBindingListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionRuntimeBindingListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionRuntimeBindingListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionRuntimeBindingRequest {
  final String? runtimeBindingId;
  final String? runtimeLocationId;
  final String hostMode;
  final String transportKind;
  final String providerBindingId;
  final String modelId;
  final String providerId;
  final String? providerSessionId;
  final String? providerSessionTreeId;
  final String? providerParentSessionId;
  final String? providerForkedFromSessionId;
  final String requestedAt;

  CreateAgentSessionRuntimeBindingRequest({
    this.runtimeBindingId,
    this.runtimeLocationId,
    required this.hostMode,
    required this.transportKind,
    required this.providerBindingId,
    required this.modelId,
    required this.providerId,
    this.providerSessionId,
    this.providerSessionTreeId,
    this.providerParentSessionId,
    this.providerForkedFromSessionId,
    required this.requestedAt
  });

  factory CreateAgentSessionRuntimeBindingRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionRuntimeBindingRequest(
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      hostMode: (() {
        final value = json['hostMode']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.hostMode is required');
        }
        return value;
      })(),
      transportKind: (() {
        final value = json['transportKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.transportKind is required');
        }
        return value;
      })(),
      providerBindingId: (() {
        final value = json['providerBindingId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.providerBindingId is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.modelId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.providerId is required');
        }
        return value;
      })(),
      providerSessionId: json['providerSessionId']?.toString(),
      providerSessionTreeId: json['providerSessionTreeId']?.toString(),
      providerParentSessionId: json['providerParentSessionId']?.toString(),
      providerForkedFromSessionId: json['providerForkedFromSessionId']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeBindingId': runtimeBindingId,
      'runtimeLocationId': runtimeLocationId,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'providerSessionId': providerSessionId,
      'providerSessionTreeId': providerSessionTreeId,
      'providerParentSessionId': providerParentSessionId,
      'providerForkedFromSessionId': providerForkedFromSessionId,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentSessionRuntimeBindingRequest {
  final String? runtimeLocationId;
  final bool? clearRuntimeLocation;
  final String? hostMode;
  final String? transportKind;
  final String? providerBindingId;
  final String? modelId;
  final String? providerId;
  final String? providerSessionId;
  final String? providerSessionTreeId;
  final String? providerParentSessionId;
  final String? providerForkedFromSessionId;
  final String expectedVersion;
  final String requestedAt;

  UpdateAgentSessionRuntimeBindingRequest({
    this.runtimeLocationId,
    this.clearRuntimeLocation,
    this.hostMode,
    this.transportKind,
    this.providerBindingId,
    this.modelId,
    this.providerId,
    this.providerSessionId,
    this.providerSessionTreeId,
    this.providerParentSessionId,
    this.providerForkedFromSessionId,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory UpdateAgentSessionRuntimeBindingRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentSessionRuntimeBindingRequest(
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      clearRuntimeLocation: json['clearRuntimeLocation'] is bool ? json['clearRuntimeLocation'] : null,
      hostMode: json['hostMode']?.toString(),
      transportKind: json['transportKind']?.toString(),
      providerBindingId: json['providerBindingId']?.toString(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      providerSessionId: json['providerSessionId']?.toString(),
      providerSessionTreeId: json['providerSessionTreeId']?.toString(),
      providerParentSessionId: json['providerParentSessionId']?.toString(),
      providerForkedFromSessionId: json['providerForkedFromSessionId']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentSessionRuntimeBindingRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentSessionRuntimeBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeLocationId': runtimeLocationId,
      'clearRuntimeLocation': clearRuntimeLocation,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'providerSessionId': providerSessionId,
      'providerSessionTreeId': providerSessionTreeId,
      'providerParentSessionId': providerParentSessionId,
      'providerForkedFromSessionId': providerForkedFromSessionId,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class ChangeAgentSessionRuntimeBindingStatusRequest {
  final String? reason;
  final String expectedVersion;
  final String requestedAt;

  ChangeAgentSessionRuntimeBindingStatusRequest({
    this.reason,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ChangeAgentSessionRuntimeBindingStatusRequest.fromJson(Map<String, dynamic> json) {
    return ChangeAgentSessionRuntimeBindingStatusRequest(
      reason: json['reason']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ChangeAgentSessionRuntimeBindingStatusRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ChangeAgentSessionRuntimeBindingStatusRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'reason': reason,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class MediaToolDirectoryEntry {
  final String toolId;
  final String category;
  final String name;
  final String displayName;
  final String? version;
  final String? description;
  final Map<String, dynamic>? inputSchema;
  final Map<String, dynamic>? outputSchema;
  final String? sideEffectLevel;
  final List<String>? policyCategories;
  final int? timeoutMs;
  final String? availability;
  final bool enabled;
  final bool? saveToDriveDefault;
  final bool? configured;

  MediaToolDirectoryEntry({
    required this.toolId,
    required this.category,
    required this.name,
    required this.displayName,
    this.version,
    this.description,
    this.inputSchema,
    this.outputSchema,
    this.sideEffectLevel,
    this.policyCategories,
    this.timeoutMs,
    this.availability,
    required this.enabled,
    this.saveToDriveDefault,
    this.configured
  });

  factory MediaToolDirectoryEntry.fromJson(Map<String, dynamic> json) {
    return MediaToolDirectoryEntry(
      toolId: (() {
        final value = json['toolId']?.toString();
        if (value == null) {
          throw FormatException('MediaToolDirectoryEntry.toolId is required');
        }
        return value;
      })(),
      category: (() {
        final value = json['category']?.toString();
        if (value == null) {
          throw FormatException('MediaToolDirectoryEntry.category is required');
        }
        return value;
      })(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('MediaToolDirectoryEntry.name is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('MediaToolDirectoryEntry.displayName is required');
        }
        return value;
      })(),
      version: json['version']?.toString(),
      description: json['description']?.toString(),
      inputSchema: _sdkworkAsMap(json['inputSchema']),
      outputSchema: _sdkworkAsMap(json['outputSchema']),
      sideEffectLevel: json['sideEffectLevel']?.toString(),
      policyCategories: (() {
        final list = _sdkworkAsList(json['policyCategories']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      timeoutMs: json['timeoutMs'] is int ? json['timeoutMs'] : null,
      availability: json['availability']?.toString(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('MediaToolDirectoryEntry.enabled is required');
        }
        return value;
      })(),
      saveToDriveDefault: json['saveToDriveDefault'] is bool ? json['saveToDriveDefault'] : null,
      configured: json['configured'] is bool ? json['configured'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'toolId': toolId,
      'category': category,
      'name': name,
      'displayName': displayName,
      'version': version,
      'description': description,
      'inputSchema': inputSchema,
      'outputSchema': outputSchema,
      'sideEffectLevel': sideEffectLevel,
      'policyCategories': policyCategories?.map((item) => item).toList(),
      'timeoutMs': timeoutMs,
      'availability': availability,
      'enabled': enabled,
      'saveToDriveDefault': saveToDriveDefault,
      'configured': configured,
    };
  }
}

class MediaToolInvokeBody {
  final Map<String, dynamic> arguments;
  final bool? saveToDrive;

  MediaToolInvokeBody({
    required this.arguments,
    this.saveToDrive
  });

  factory MediaToolInvokeBody.fromJson(Map<String, dynamic> json) {
    return MediaToolInvokeBody(
      arguments: (() {
        final map = _sdkworkAsMap(json['arguments']);
        if (map == null) {
          throw FormatException('MediaToolInvokeBody.arguments is required');
        }
        return map;
      })(),
      saveToDrive: json['saveToDrive'] is bool ? json['saveToDrive'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'arguments': arguments,
      'saveToDrive': saveToDrive,
    };
  }
}

class MediaToolInvokeResponse {
  final String toolCallId;
  final String status;
  final Map<String, dynamic> output;
  final String? error;
  final DriveAssetView? driveAsset;

  MediaToolInvokeResponse({
    required this.toolCallId,
    required this.status,
    required this.output,
    this.error,
    this.driveAsset
  });

  factory MediaToolInvokeResponse.fromJson(Map<String, dynamic> json) {
    return MediaToolInvokeResponse(
      toolCallId: (() {
        final value = json['toolCallId']?.toString();
        if (value == null) {
          throw FormatException('MediaToolInvokeResponse.toolCallId is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('MediaToolInvokeResponse.status is required');
        }
        return value;
      })(),
      output: (() {
        final map = _sdkworkAsMap(json['output']);
        if (map == null) {
          throw FormatException('MediaToolInvokeResponse.output is required');
        }
        return map;
      })(),
      error: json['error']?.toString(),
      driveAsset: (() {
        final map = _sdkworkAsMap(json['driveAsset']);
        return map == null ? null : DriveAssetView.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'toolCallId': toolCallId,
      'status': status,
      'output': output,
      'error': error,
      'driveAsset': driveAsset?.toJson(),
    };
  }
}

class DriveAssetView {
  final String spaceId;
  final String nodeId;
  final String driveUri;

  DriveAssetView({
    required this.spaceId,
    required this.nodeId,
    required this.driveUri
  });

  factory DriveAssetView.fromJson(Map<String, dynamic> json) {
    return DriveAssetView(
      spaceId: (() {
        final value = json['spaceId']?.toString();
        if (value == null) {
          throw FormatException('DriveAssetView.spaceId is required');
        }
        return value;
      })(),
      nodeId: (() {
        final value = json['nodeId']?.toString();
        if (value == null) {
          throw FormatException('DriveAssetView.nodeId is required');
        }
        return value;
      })(),
      driveUri: (() {
        final value = json['driveUri']?.toString();
        if (value == null) {
          throw FormatException('DriveAssetView.driveUri is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'spaceId': spaceId,
      'nodeId': nodeId,
      'driveUri': driveUri,
    };
  }
}

class ToolAssetView {
  final String toolId;
  final String toolCallId;
  final String mediaKind;
  final String driveSpaceId;
  final String driveNodeId;
  final String driveUri;
  final String? sourceUrl;
  final String? createdAt;

  ToolAssetView({
    required this.toolId,
    required this.toolCallId,
    required this.mediaKind,
    required this.driveSpaceId,
    required this.driveNodeId,
    required this.driveUri,
    this.sourceUrl,
    this.createdAt
  });

  factory ToolAssetView.fromJson(Map<String, dynamic> json) {
    return ToolAssetView(
      toolId: (() {
        final value = json['toolId']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.toolId is required');
        }
        return value;
      })(),
      toolCallId: (() {
        final value = json['toolCallId']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.toolCallId is required');
        }
        return value;
      })(),
      mediaKind: (() {
        final value = json['mediaKind']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.mediaKind is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.driveSpaceId is required');
        }
        return value;
      })(),
      driveNodeId: (() {
        final value = json['driveNodeId']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.driveNodeId is required');
        }
        return value;
      })(),
      driveUri: (() {
        final value = json['driveUri']?.toString();
        if (value == null) {
          throw FormatException('ToolAssetView.driveUri is required');
        }
        return value;
      })(),
      sourceUrl: json['sourceUrl']?.toString(),
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'toolId': toolId,
      'toolCallId': toolCallId,
      'mediaKind': mediaKind,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'driveUri': driveUri,
      'sourceUrl': sourceUrl,
      'createdAt': createdAt,
    };
  }
}

class MediaToolConfigurationBody {
  final bool enabled;
  final bool? saveToDriveDefault;
  final Map<String, dynamic>? defaultArguments;
  final int? expectedVersion;

  MediaToolConfigurationBody({
    required this.enabled,
    this.saveToDriveDefault,
    this.defaultArguments,
    this.expectedVersion
  });

  factory MediaToolConfigurationBody.fromJson(Map<String, dynamic> json) {
    return MediaToolConfigurationBody(
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('MediaToolConfigurationBody.enabled is required');
        }
        return value;
      })(),
      saveToDriveDefault: json['saveToDriveDefault'] is bool ? json['saveToDriveDefault'] : null,
      defaultArguments: _sdkworkAsMap(json['defaultArguments']),
      expectedVersion: json['expectedVersion'] is int ? json['expectedVersion'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'enabled': enabled,
      'saveToDriveDefault': saveToDriveDefault,
      'defaultArguments': defaultArguments,
      'expectedVersion': expectedVersion,
    };
  }
}

class FieldError {
  final String field;
  final String message;
  final int? code;

  FieldError({
    required this.field,
    required this.message,
    this.code
  });

  factory FieldError.fromJson(Map<String, dynamic> json) {
    return FieldError(
      field: (() {
        final value = json['field']?.toString();
        if (value == null) {
          throw FormatException('FieldError.field is required');
        }
        return value;
      })(),
      message: (() {
        final value = json['message']?.toString();
        if (value == null) {
          throw FormatException('FieldError.message is required');
        }
        return value;
      })(),
      code: json['code'] is int ? json['code'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'field': field,
      'message': message,
      'code': code,
    };
  }
}

class ProblemDetail {
  final String type;
  final String title;
  final int status;
  final String? detail;
  final String? instance;
  final int code;
  final String traceId;
  final List<FieldError>? errors;

  ProblemDetail({
    required this.type,
    required this.title,
    required this.status,
    this.detail,
    this.instance,
    required this.code,
    required this.traceId,
    this.errors
  });

  factory ProblemDetail.fromJson(Map<String, dynamic> json) {
    return ProblemDetail(
      type: (() {
        final value = json['type']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.type is required');
        }
        return value;
      })(),
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.title is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status'];
        if (value is! int) {
          throw FormatException('ProblemDetail.status is required');
        }
        return value;
      })(),
      detail: json['detail']?.toString(),
      instance: json['instance']?.toString(),
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ProblemDetail.code is required');
        }
        return value;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.traceId is required');
        }
        return value;
      })(),
      errors: (() {
        final list = _sdkworkAsList(json['errors']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : FieldError.fromJson(map);
      })())
            .whereType<FieldError>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'type': type,
      'title': title,
      'status': status,
      'detail': detail,
      'instance': instance,
      'code': code,
      'traceId': traceId,
      'errors': errors?.map((item) => item.toJson()).toList(),
    };
  }
}
